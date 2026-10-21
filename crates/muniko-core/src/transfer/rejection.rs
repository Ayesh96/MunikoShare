use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::messages::transfer as msg;

/// Why a transfer request was refused; maps one-to-one onto an HTTP status.
#[derive(Debug)]
pub enum Rejection {
    BadRequest(&'static str),
    Forbidden(&'static str),
    NotFound(&'static str),
    Conflict(&'static str),
    OfferTimedOut,
    ChecksumMismatch,
    Internal(&'static str),
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        let (status, text) = match self {
            Rejection::BadRequest(text) => (StatusCode::BAD_REQUEST, text),
            Rejection::Forbidden(text) => (StatusCode::FORBIDDEN, text),
            Rejection::NotFound(text) => (StatusCode::NOT_FOUND, text),
            Rejection::Conflict(text) => (StatusCode::CONFLICT, text),
            Rejection::OfferTimedOut => (StatusCode::REQUEST_TIMEOUT, msg::OFFER_TIMED_OUT),
            Rejection::ChecksumMismatch => (StatusCode::UNPROCESSABLE_ENTITY, msg::CHECKSUM_MISMATCH),
            Rejection::Internal(text) => (StatusCode::INTERNAL_SERVER_ERROR, text),
        };
        (status, text).into_response()
    }
}
