use std::net::IpAddr;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::messages::server as msg;

/// Who is on the other end of a connection; injected into every request as an extension.
#[derive(Debug, Clone)]
pub struct RequestPeer {
    pub ip: IpAddr,
    /// Fingerprint of the presented client certificate, if any.
    pub device_id: Option<String>,
}

/// Extractor for routes that need a device that presented a client certificate.
#[derive(Debug, Clone)]
pub struct AuthenticatedPeer {
    pub ip: IpAddr,
    pub device_id: String,
}

/// Rejection for a missing client certificate (401) or missing connection details (500).
#[derive(Debug)]
pub enum PeerRejection {
    Unauthenticated,
    Missing,
}

impl IntoResponse for PeerRejection {
    fn into_response(self) -> Response {
        match self {
            PeerRejection::Unauthenticated => (StatusCode::UNAUTHORIZED, msg::AUTH_REQUIRED).into_response(),
            PeerRejection::Missing => (StatusCode::INTERNAL_SERVER_ERROR, msg::PEER_MISSING).into_response(),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedPeer {
    type Rejection = PeerRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let peer = parts.extensions.get::<RequestPeer>().ok_or(PeerRejection::Missing)?;
        let device_id = peer.device_id.clone().ok_or(PeerRejection::Unauthenticated)?;
        Ok(Self { ip: peer.ip, device_id })
    }
}
