//! Redeem a QR token for a session, check it, end it.

use axum::extract::{Extension, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use super::auth::PhoneAuth;
use super::cookie::{clear_cookie, session_cookie};
use super::service::{MobileService, RedeemError};
use super::session::PhoneSession;
use crate::messages::mobile as msg;
use crate::server::RequestPeer;

#[derive(Deserialize)]
pub struct RedeemBody {
    token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionReply {
    device_name: String,
    session_id: String,
}

pub async fn redeem(State(service): State<MobileService>, Extension(peer): Extension<RequestPeer>, headers: HeaderMap, Json(body): Json<RedeemBody>) -> Response {
    let agent = headers.get(header::USER_AGENT).and_then(|value| value.to_str().ok()).unwrap_or_default();
    match service.redeem(&body.token, peer.ip, agent) {
        Ok(session) => session_reply(&service, &session),
        Err(RedeemError::Throttled) => (StatusCode::TOO_MANY_REQUESTS, msg::TOO_MANY_ATTEMPTS).into_response(),
        Err(RedeemError::Invalid) => (StatusCode::UNAUTHORIZED, msg::UNAUTHORIZED).into_response(),
    }
}

pub async fn current(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth) -> Response {
    session_reply(&service, &session)
}

pub async fn logout(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth) -> Response {
    service.logout(&session.id);
    (StatusCode::NO_CONTENT, [(header::SET_COOKIE, clear_cookie())]).into_response()
}

// Re-sent on every check so the cookie slides together with the server-side session.
fn session_reply(service: &MobileService, session: &PhoneSession) -> Response {
    let cookie = session_cookie(&session.id, service.session_ttl_secs());
    let reply = SessionReply { device_name: service.device_name().to_string(), session_id: session.id.clone() };
    ([(header::SET_COOKIE, cookie)], Json(reply)).into_response()
}
