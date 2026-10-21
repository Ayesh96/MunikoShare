use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};

use super::local_device::LocalDevice;
use crate::constants::info::ROUTE_INFO;
use crate::messages::info as msg;
use crate::server::{AuthenticatedPeer, RouteProvider};

/// Plugs `GET /api/v1/info` into the shared server.
pub struct InfoRoutes {
    local: LocalDevice,
}

impl InfoRoutes {
    pub fn new(local: LocalDevice) -> Self {
        Self { local }
    }
}

impl RouteProvider for InfoRoutes {
    fn routes(&self) -> Router {
        Router::new().route(ROUTE_INFO, get(info)).with_state(self.local.clone())
    }
}

async fn info(_peer: AuthenticatedPeer, State(local): State<LocalDevice>) -> Response {
    match local.snapshot() {
        Some(device) => Json(device).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR, msg::LOCAL_DEVICE_LOCKED).into_response(),
    }
}
