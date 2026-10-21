use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;

use super::access::ClipboardAccess;
use super::content::ClipboardContent;
use super::history::HistorySource;
use super::shared::ClipboardShared;
use super::wire::ClipboardPayload;
use crate::constants::clipboard::{MAX_REQUEST_BODY_BYTES, ROUTE_CLIPBOARD};
use crate::error::MunikoError;
use crate::events::{EventBus, NodeEvent};
use crate::messages::clipboard as msg;
use crate::server::{AuthenticatedPeer, RouteProvider};
use crate::trust::TrustStore;

struct RouteState {
    access: Arc<dyn ClipboardAccess>,
    trust: Arc<dyn TrustStore>,
    events: EventBus,
    shared: ClipboardShared,
}

/// Plugs `POST /api/v1/clipboard` into the shared server; trusted peers only.
pub struct ClipboardRoutes {
    state: Arc<RouteState>,
}

impl ClipboardRoutes {
    /// Standalone routes with private loop-guard and history state.
    pub fn new(access: Arc<dyn ClipboardAccess>, trust: Arc<dyn TrustStore>, events: EventBus) -> Self {
        let shared = ClipboardShared::new(access.clone());
        Self::with_shared(access, trust, events, shared)
    }

    /// Routes that share loop-guard and history state with the watcher.
    pub fn with_shared(access: Arc<dyn ClipboardAccess>, trust: Arc<dyn TrustStore>, events: EventBus, shared: ClipboardShared) -> Self {
        Self { state: Arc::new(RouteState { access, trust, events, shared }) }
    }
}

impl RouteProvider for ClipboardRoutes {
    fn routes(&self) -> Router {
        Router::new()
            .route(ROUTE_CLIPBOARD, post(receive))
            .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
            .with_state(self.state.clone())
    }
}

async fn receive(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Json(body): Json<Value>) -> Response {
    if !state.trust.is_trusted(&peer.device_id) {
        return (StatusCode::FORBIDDEN, msg::NOT_TRUSTED).into_response();
    }
    let (content, hash) = match accept(body) {
        Ok(accepted) => accepted,
        Err(rejection) => return rejection,
    };
    // Remember before writing so the watcher cannot see the write first and echo it.
    state.shared.hashes.remember_received(&hash);
    match write_clipboard(&state, content.clone()).await {
        Ok(()) => {
            tracing::info!(from = %peer.device_id, "{}", received_log(&content));
            state.shared.history.record(HistorySource::Device { device_id: peer.device_id.clone() }, content.clone());
            state.events.publish(received_event(peer.device_id, content));
            StatusCode::NO_CONTENT.into_response()
        }
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

/// Parses and validates a body; the error is the ready-made rejection response.
fn accept(body: Value) -> Result<(ClipboardContent, String), Response> {
    let payload = ClipboardPayload::parse(body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())?;
    let claimed = payload.origin_hash.clone();
    let content = payload.into_content().map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())?;
    if let Some(problem) = content.problem() {
        return Err((problem.status(), problem.message()).into_response());
    }
    let hash = content.hash();
    if claimed.is_some_and(|claimed| claimed != hash) {
        return Err((StatusCode::BAD_REQUEST, msg::HASH_MISMATCH).into_response());
    }
    Ok((content, hash))
}

fn received_event(from_device_id: String, content: ClipboardContent) -> NodeEvent {
    match content {
        ClipboardContent::Text(text) => NodeEvent::ClipboardReceived { from_device_id, text },
        ClipboardContent::Image { width, height, .. } => NodeEvent::ClipboardImageReceived { from_device_id, width, height },
    }
}

fn received_log(content: &ClipboardContent) -> &'static str {
    match content {
        ClipboardContent::Text(_) => msg::RECEIVED_LOG,
        ClipboardContent::Image { .. } => msg::IMAGE_RECEIVED_LOG,
    }
}

async fn write_clipboard(state: &RouteState, content: ClipboardContent) -> Result<(), MunikoError> {
    let access = state.access.clone();
    tokio::task::spawn_blocking(move || access.write_content(&content))
        .await
        .map_err(|e| MunikoError::InvalidState(format!("{}: {e}", msg::TASK_FAILED)))?
}
