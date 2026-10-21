//! Turns a peer's HTTP reply into transfer errors the sender can show.

use reqwest::{Response, StatusCode};

use crate::error::MunikoError;
use crate::messages::transfer as msg;

pub fn transfer_error(context: &str, detail: impl std::fmt::Display) -> MunikoError {
    MunikoError::Transfer(format!("{context}: {detail}"))
}

pub fn into_transfer_error(error: MunikoError) -> MunikoError {
    match error {
        MunikoError::Network(detail) => transfer_error(msg::SEND_FAILED, detail),
        other => other,
    }
}

/// Success responses pass through; the peer's refusal becomes a clear error.
pub async fn ensure_accepted(response: Response) -> Result<Response, MunikoError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let reason = response.text().await.unwrap_or_default();
    Err(match status {
        StatusCode::FORBIDDEN if reason == msg::DECLINED => MunikoError::Transfer(msg::PEER_DECLINED.to_string()),
        StatusCode::FORBIDDEN => MunikoError::Transfer(msg::PEER_NOT_TRUSTING.to_string()),
        StatusCode::REQUEST_TIMEOUT => MunikoError::Transfer(msg::PEER_TIMED_OUT.to_string()),
        _ => transfer_error(msg::PEER_REJECTED, format!("{status} {reason}")),
    })
}
