use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use async_trait::async_trait;
use axum::middleware;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use tokio::task::JoinHandle;

use super::acceptor::PeerAcceptor;
use super::middleware::require_local_peer;
use super::route_provider::RouteProvider;
use super::tls::server_config;
use crate::constants::app::DEFAULT_PORT;
use crate::constants::server::{BIND_HOST, SHUTDOWN_GRACE};
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::server as msg;

type ServerHandle = Handle<SocketAddr>;

struct RunningServer {
    port: u16,
    handle: ServerHandle,
    task: JoinHandle<std::io::Result<()>>,
}

/// The one HTTPS server; feature modules plug their routes in as providers.
pub struct HttpServer {
    identity: Arc<Identity>,
    providers: Vec<Arc<dyn RouteProvider>>,
    state: LifecycleState,
    running: Option<RunningServer>,
}

impl HttpServer {
    pub fn new(identity: Arc<Identity>, providers: Vec<Arc<dyn RouteProvider>>) -> Self {
        Self { identity, providers, state: LifecycleState::Created, running: None }
    }

    /// The port actually bound, once running.
    pub fn port(&self) -> Option<u16> {
        self.running.as_ref().map(|running| running.port)
    }

    fn router(&self) -> Router {
        let merged = self.providers.iter().fold(Router::new(), |router, provider| router.merge(provider.routes()));
        merged.layer(middleware::from_fn(require_local_peer))
    }
}

#[async_trait]
impl Lifecycle for HttpServer {
    async fn start(&mut self) -> Result<()> {
        if self.state == LifecycleState::Running {
            return Err(MunikoError::InvalidState(format!("{:?}", self.state)));
        }
        let listener = bind_listener()?;
        let port = listener.local_addr().map_err(bind_error)?.port();
        let tls = RustlsConfig::from_config(Arc::new(server_config(&self.identity)?));
        let handle = ServerHandle::new();
        let server = axum_server::from_tcp(listener)
            .map_err(bind_error)?
            .acceptor(PeerAcceptor::new(tls))
            .handle(handle.clone());
        let app = self.router().into_make_service();
        let task = tokio::spawn(async move { server.serve(app).await });
        self.running = Some(RunningServer { port, handle, task });
        self.state = LifecycleState::Running;
        tracing::info!(port, "{}", msg::SERVER_STARTED);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(running) = self.running.take() {
            running.handle.graceful_shutdown(Some(SHUTDOWN_GRACE));
            running
                .task
                .await
                .map_err(|e| MunikoError::Server(format!("{}: {e}", msg::SERVER_TASK_FAILED)))??;
        }
        self.state = LifecycleState::Stopped;
        tracing::info!("{}", msg::SERVER_STOPPED);
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

fn bind_error(error: std::io::Error) -> MunikoError {
    MunikoError::Server(format!("{}: {error}", msg::BIND_FAILED))
}

// Prefers the well-known port so peers can guess it, else lets the OS pick.
fn bind_listener() -> Result<TcpListener> {
    let listener = TcpListener::bind((BIND_HOST, DEFAULT_PORT)).or_else(|_| {
        tracing::warn!("{}", msg::PORT_FALLBACK);
        TcpListener::bind((BIND_HOST, 0))
    });
    let listener = listener.map_err(bind_error)?;
    listener.set_nonblocking(true).map_err(bind_error)?;
    Ok(listener)
}
