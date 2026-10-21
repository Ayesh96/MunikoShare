use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};

use crate::util::time::unix_now;
use super::code::{PendingBook, Verdict};
use super::profile::{ConfirmBody, PairingProfile};
use crate::constants::app::PROTOCOL_VERSION;
use crate::constants::server::MAX_SMALL_JSON_BYTES;
use crate::constants::pairing::{ROUTE_CONFIRM, ROUTE_REQUEST};
use crate::device::DeviceInfo;
use crate::events::{EventBus, NodeEvent};
use crate::messages::pairing as msg;
use crate::server::{AuthenticatedPeer, RouteProvider};
use crate::trust::{TrustStore, TrustedDevice};

struct RouteState {
    book: Arc<PendingBook>,
    store: Arc<dyn TrustStore>,
    events: EventBus,
    local: PairingProfile,
}

/// Plugs the pairing endpoints into the shared server.
pub struct PairingRoutes {
    state: Arc<RouteState>,
}

impl PairingRoutes {
    pub fn new(book: Arc<PendingBook>, store: Arc<dyn TrustStore>, events: EventBus, local: PairingProfile) -> Self {
        Self { state: Arc::new(RouteState { book, store, events, local }) }
    }
}

impl RouteProvider for PairingRoutes {
    fn routes(&self) -> Router {
        Router::new()
            .route(ROUTE_REQUEST, post(request))
            .route(ROUTE_CONFIRM, post(confirm))
            .layer(DefaultBodyLimit::max(MAX_SMALL_JSON_BYTES))
            .with_state(self.state.clone())
    }
}

fn requester_info(peer: &AuthenticatedPeer, profile: &PairingProfile) -> DeviceInfo {
    DeviceInfo {
        id: peer.device_id.clone(),
        name: profile.name.clone(),
        os: profile.os.clone(),
        addresses: vec![peer.ip],
        port: 0,
        protocol_version: PROTOCOL_VERSION,
    }
}

async fn request(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Json(profile): Json<PairingProfile>) -> Response {
    let code = state.book.begin(&peer.device_id, profile.clone());
    tracing::info!("{} from {}", msg::LOG_REQUESTED, peer.ip);
    state.events.publish(NodeEvent::PairingRequested { device: requester_info(&peer, &profile), code });
    StatusCode::ACCEPTED.into_response()
}

async fn confirm(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Json(body): Json<ConfirmBody>) -> Response {
    match state.book.verify(&peer.device_id, &body.code) {
        Verdict::Accepted(profile) => complete(&state, &peer.device_id, profile),
        Verdict::WrongCode => (StatusCode::FORBIDDEN, msg::WRONG_CODE).into_response(),
        Verdict::Expired => {
            fail(&state, &peer.device_id, msg::CODE_EXPIRED);
            (StatusCode::GONE, msg::CODE_EXPIRED).into_response()
        }
        Verdict::NoPending => (StatusCode::NOT_FOUND, msg::NO_PENDING).into_response(),
    }
}

fn fail(state: &RouteState, device_id: &str, reason: &str) {
    state.events.publish(NodeEvent::PairingFailed { device_id: device_id.to_string(), reason: reason.to_string() });
}

fn complete(state: &RouteState, device_id: &str, profile: PairingProfile) -> Response {
    let device = TrustedDevice { id: device_id.to_string(), name: profile.name.clone(), os: profile.os, paired_at: unix_now() };
    if let Err(error) = state.store.add(device) {
        fail(state, device_id, &error.to_string());
        return (StatusCode::INTERNAL_SERVER_ERROR, msg::STORE_FAILED).into_response();
    }
    state.events.publish(NodeEvent::PairingCompleted { device_id: device_id.to_string(), device_name: profile.name });
    Json(state.local.clone()).into_response()
}
