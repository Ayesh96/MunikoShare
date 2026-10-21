//! The phone's inbox: list items and stream one with Range support.

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::auth::PhoneAuth;
use super::file_response::serve_file;
use super::service::MobileService;
use crate::messages::mobile as msg;

pub async fn list(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth) -> Response {
    Json(service.inbox(&session.id)).into_response()
}

pub async fn download(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth, Path(id): Path<String>, headers: HeaderMap) -> Response {
    let Some(item) = service.inbox_item(&session.id, &id) else { return (StatusCode::NOT_FOUND, msg::NOT_FOUND).into_response() };
    let range = headers.get(header::RANGE).and_then(|value| value.to_str().ok());
    match serve_file(&item.path, &item.name, range, ()).await {
        Ok(response) => response,
        Err(status) => status.into_response(),
    }
}
