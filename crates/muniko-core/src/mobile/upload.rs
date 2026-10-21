//! Streams a phone's upload into the download folder.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::StreamExt;
use percent_encoding::percent_decode_str;
use serde::Serialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::auth::PhoneAuth;
use super::service::MobileService;
use crate::constants::mobile::FILE_NAME_HEADER;
use crate::messages::mobile as msg;
use crate::util::fs::{claim_unique_path, ensure_dir};
use crate::util::sanitize::sanitize_file_name;

pub(super) enum UploadError {
    Empty,
    TooLarge,
    Failed,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        match self {
            Self::Empty => (StatusCode::BAD_REQUEST, msg::EMPTY_UPLOAD),
            Self::TooLarge => (StatusCode::PAYLOAD_TOO_LARGE, msg::UPLOAD_TOO_LARGE),
            Self::Failed => (StatusCode::INTERNAL_SERVER_ERROR, msg::UPLOAD_FAILED),
        }
        .into_response()
    }
}

#[derive(Serialize)]
struct UploadReply {
    name: String,
}

pub async fn upload(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth, headers: HeaderMap, body: Body) -> Response {
    let Some(name) = file_name(&headers) else { return (StatusCode::BAD_REQUEST, msg::BAD_FILE_NAME).into_response() };
    let limit = service.max_upload_bytes();
    if declared_length(&headers).is_some_and(|length| length > limit) {
        return UploadError::TooLarge.into_response();
    }
    match store(&service.download_dir(), &name, body, limit).await {
        Ok(path) => {
            let saved = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or(name);
            service.record_upload(&session.id, path);
            (StatusCode::CREATED, Json(UploadReply { name: saved })).into_response()
        }
        Err(error) => error.into_response(),
    }
}

pub(super) fn file_name(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(FILE_NAME_HEADER)?.to_str().ok()?;
    sanitize_file_name(&percent_decode_str(raw).decode_utf8().ok()?)
}

pub(super) fn declared_length(headers: &HeaderMap) -> Option<u64> {
    headers.get(header::CONTENT_LENGTH)?.to_str().ok()?.parse().ok()
}

// A failed or oversized upload leaves no file behind.
async fn store(dir: &Path, name: &str, body: Body, limit: u64) -> Result<PathBuf, UploadError> {
    ensure_dir(dir).await.map_err(|_| UploadError::Failed)?;
    let path = claim_unique_path(dir, name).await.map_err(|_| UploadError::Failed)?;
    let outcome = write_body(&path, body, limit).await;
    if outcome.is_err() {
        let _ = fs::remove_file(&path).await;
    }
    outcome.map(|()| path)
}

pub(super) async fn write_body(path: &Path, body: Body, limit: u64) -> Result<(), UploadError> {
    let mut file = fs::File::create(path).await.map_err(|_| UploadError::Failed)?;
    let mut stream = body.into_data_stream();
    let mut written = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| UploadError::Failed)?;
        written += chunk.len() as u64;
        if written > limit {
            return Err(UploadError::TooLarge);
        }
        file.write_all(&chunk).await.map_err(|_| UploadError::Failed)?;
    }
    file.flush().await.map_err(|_| UploadError::Failed)?;
    if written == 0 {
        return Err(UploadError::Empty);
    }
    Ok(())
}
