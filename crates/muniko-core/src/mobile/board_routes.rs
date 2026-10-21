//! The phone's view of the Network Board: list, post text, post a file, download a file.

use std::path::{Path as FsPath, PathBuf};

use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use super::auth::PhoneAuth;
use super::board_errors::board_error;
use super::file_response::serve_file;
use super::service::MobileService;
use super::text_routes::TextBody;
use super::upload::{declared_length, file_name, write_body, UploadError};
use crate::board::{Board, BoardItemView, OpenedFile, ViewKind};
use crate::constants::mobile::{MAX_TEXT_BYTES, UPLOAD_TEMP_PREFIX};
use crate::error::Result;
use crate::messages::mobile as msg;
use crate::util::fs::ensure_dir;

#[derive(Serialize)]
struct BoardReply {
    board: Option<Board>,
    items: Vec<BoardItemView>,
}

pub async fn list(State(service): State<MobileService>, _auth: PhoneAuth) -> Response {
    let reply = service.board().board().and_then(|board| {
        let items = if board.is_some() { service.board().items()? } else { Vec::new() };
        Ok(BoardReply { board, items })
    });
    match reply {
        Ok(reply) => Json(reply).into_response(),
        Err(error) => board_error(&error),
    }
}

pub async fn post_text(State(service): State<MobileService>, _auth: PhoneAuth, body: Bytes) -> Response {
    let Ok(TextBody { text }) = serde_json::from_slice(&body) else { return (StatusCode::BAD_REQUEST, msg::BAD_TEXT_BODY).into_response() };
    if text.len() > MAX_TEXT_BYTES {
        return (StatusCode::PAYLOAD_TOO_LARGE, msg::TEXT_TOO_LARGE).into_response();
    }
    created(service.board().post_text(&text))
}

pub async fn post_file(State(service): State<MobileService>, _auth: PhoneAuth, headers: HeaderMap, body: Body) -> Response {
    let Some(name) = file_name(&headers) else { return (StatusCode::BAD_REQUEST, msg::BAD_FILE_NAME).into_response() };
    let limit = service.max_upload_bytes();
    if declared_length(&headers).is_some_and(|length| length > limit) {
        return UploadError::TooLarge.into_response();
    }
    let dir = std::env::temp_dir().join(format!("{UPLOAD_TEMP_PREFIX}{}", Uuid::new_v4().simple()));
    let response = stage_and_post(&service, &dir, &name, body, limit).await;
    let _ = tokio::fs::remove_dir_all(&dir).await;
    response
}

// The board copies the file into its cache, so the staged copy is always removed by the caller.
async fn stage_and_post(service: &MobileService, dir: &FsPath, name: &str, body: Body, limit: u64) -> Response {
    let path: PathBuf = dir.join(name);
    if ensure_dir(dir).await.is_err() {
        return UploadError::Failed.into_response();
    }
    if let Err(error) = write_body(&path, body, limit).await {
        return error.into_response();
    }
    created(service.board().post_file(&path).await)
}

pub async fn download(State(service): State<MobileService>, _auth: PhoneAuth, Path(id): Path<String>, headers: HeaderMap) -> Response {
    let OpenedFile { path, view, lease } = match service.board().open_file(&id).await {
        Ok(opened) => opened,
        Err(error) => return board_error(&error),
    };
    let ViewKind::File { name, .. } = view.kind else { return (StatusCode::NOT_FOUND, msg::NOT_FOUND).into_response() };
    let range = headers.get(header::RANGE).and_then(|value| value.to_str().ok());
    serve_file(&path, &name, range, lease).await.unwrap_or_else(IntoResponse::into_response)
}

fn created(posted: Result<BoardItemView>) -> Response {
    match posted {
        Ok(view) => (StatusCode::CREATED, Json(view)).into_response(),
        Err(error) => board_error(&error),
    }
}
