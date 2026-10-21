//! Extractor that admits only requests carrying a valid phone session cookie.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::cookie::read_session_id;
use super::service::MobileService;
use super::session::PhoneSession;
use crate::messages::mobile as msg;

pub struct PhoneAuth(pub PhoneSession);

impl FromRequestParts<MobileService> for PhoneAuth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, service: &MobileService) -> Result<Self, Self::Rejection> {
        let session = read_session_id(&parts.headers).and_then(|id| service.authenticate(&id));
        session.map(PhoneAuth).ok_or_else(|| (StatusCode::UNAUTHORIZED, msg::UNAUTHORIZED).into_response())
    }
}
