use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;

use axum::Extension;
use axum_server::accept::Accept;
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use tokio::net::TcpStream;
use tower::Layer;

use super::request_peer::RequestPeer;
use crate::util::fingerprint::fingerprint;

/// Wraps the TLS acceptor and tags every request on a connection with its RequestPeer.
#[derive(Clone)]
pub struct PeerAcceptor {
    tls: RustlsAcceptor,
}

impl PeerAcceptor {
    pub fn new(config: RustlsConfig) -> Self {
        Self { tls: RustlsAcceptor::new(config) }
    }
}

type TlsStream = <RustlsAcceptor as Accept<TcpStream, ()>>::Stream;

impl<S> Accept<TcpStream, S> for PeerAcceptor
where
    S: Send + 'static,
{
    type Stream = TlsStream;
    type Service = <Extension<RequestPeer> as Layer<S>>::Service;
    type Future = Pin<Box<dyn Future<Output = io::Result<(Self::Stream, Self::Service)>> + Send>>;

    fn accept(&self, stream: TcpStream, service: S) -> Self::Future {
        let handshake = self.tls.accept(stream, service);
        Box::pin(async move {
            let (stream, service) = handshake.await?;
            let peer = peer_of(&stream)?;
            Ok((stream, Extension(peer).layer(service)))
        })
    }
}

fn peer_of(stream: &TlsStream) -> io::Result<RequestPeer> {
    let (tcp, session) = stream.get_ref();
    let address: SocketAddr = tcp.peer_addr()?;
    let device_id = session.peer_certificates().and_then(|certs| certs.first()).map(|cert| fingerprint(cert.as_ref()));
    Ok(RequestPeer { ip: address.ip(), device_id })
}
