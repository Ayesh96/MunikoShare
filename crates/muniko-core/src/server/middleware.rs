use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Extension;

use super::ip_filter::is_local_network;
use super::request_peer::RequestPeer;
use crate::messages::server as msg;

/// Rejects requests that do not come from the local network.
pub async fn require_local_peer(Extension(peer): Extension<RequestPeer>, request: Request, next: Next) -> Response {
    if is_local_network(peer.ip) {
        return next.run(request).await;
    }
    tracing::warn!(ip = %peer.ip, "{}", msg::FOREIGN_ADDRESS_BLOCKED);
    (StatusCode::FORBIDDEN, msg::NOT_LOCAL_NETWORK).into_response()
}
