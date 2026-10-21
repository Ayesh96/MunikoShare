//! Text between phone and desktop clipboard.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use super::auth::PhoneAuth;
use super::service::MobileService;
use crate::constants::mobile::MAX_TEXT_BYTES;
use crate::messages::mobile as msg;

#[derive(Deserialize)]
pub(super) struct TextBody {
    pub(super) text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TextReply {
    text: String,
    sent_at: u64,
}

pub async fn receive(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth, body: Bytes) -> Response {
    let Ok(TextBody { text }) = serde_json::from_slice(&body) else { return (StatusCode::BAD_REQUEST, msg::BAD_TEXT_BODY).into_response() };
    if text.is_empty() {
        return (StatusCode::BAD_REQUEST, msg::EMPTY_TEXT).into_response();
    }
    if text.len() > MAX_TEXT_BYTES {
        return (StatusCode::PAYLOAD_TOO_LARGE, msg::TEXT_TOO_LARGE).into_response();
    }
    match service.accept_text(&session.id, text).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, msg::CLIPBOARD_WRITE_FAILED).into_response(),
    }
}

pub async fn latest(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth) -> Response {
    match service.latest_text(&session.id) {
        Some(shared) => Json(TextReply { text: shared.text, sent_at: shared.sent_at }).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}
