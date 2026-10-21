//! Streams a cached file to a member, honouring `Range: bytes=N-` so downloads resume.

use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use super::cache::FileCache;
use crate::constants::board::{ACCEPT_RANGES_VALUE, RANGE_UNIT_PREFIX};
use crate::constants::io::READ_BUFFER_BYTES;
use crate::messages::board as msg;

/// First and optional last byte of a single `bytes=N-` or `bytes=N-M` range.
fn parse_range(header_value: &str) -> Option<(u64, Option<u64>)> {
    let spec = header_value.trim().strip_prefix(RANGE_UNIT_PREFIX)?;
    let (start, end) = spec.split_once('-')?;
    let end = if end.is_empty() { None } else { Some(end.parse().ok()?) };
    Some((start.parse().ok()?, end))
}

pub async fn serve_file(cache: &FileCache, sha256: &str, range: Option<&str>) -> Response {
    let Some(path) = cache.path_of(sha256) else { return (StatusCode::BAD_REQUEST, msg::BAD_FILE_ID).into_response() };
    // Leased before opening so eviction cannot delete the file mid-stream.
    let lease = cache.lease(sha256);
    let Ok(mut file) = File::open(&path).await else { return (StatusCode::NOT_FOUND, msg::FILE_NOT_HELD).into_response() };
    let Ok(size) = file.metadata().await.map(|m| m.len()) else { return (StatusCode::NOT_FOUND, msg::FILE_NOT_HELD).into_response() };
    let wanted = range.map(|value| parse_range(value).ok_or(())).transpose();
    let (start, last) = match wanted {
        Ok(Some((start, end))) => (start, end.unwrap_or(u64::MAX).min(size.saturating_sub(1))),
        Ok(None) => (0, size.saturating_sub(1)),
        Err(()) => return (StatusCode::BAD_REQUEST, msg::BAD_RANGE).into_response(),
    };
    if start > 0 && start >= size {
        return (StatusCode::RANGE_NOT_SATISFIABLE, [(header::CONTENT_RANGE, format!("bytes */{size}"))], msg::BAD_RANGE).into_response();
    }
    if size > 0 && last < start {
        return (StatusCode::BAD_REQUEST, msg::BAD_RANGE).into_response();
    }
    let length = if size == 0 { 0 } else { last.saturating_sub(start) + 1 };
    if file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, msg::FILE_NOT_HELD).into_response();
    }
    cache.touch(sha256);
    let body = Body::from_stream(lease.hold(ReaderStream::with_capacity(file.take(length), READ_BUFFER_BYTES)));
    let mut response = Response::builder().header(header::CONTENT_LENGTH, length).header(header::ACCEPT_RANGES, ACCEPT_RANGES_VALUE);
    response = if range.is_some() {
        response.status(StatusCode::PARTIAL_CONTENT).header(header::CONTENT_RANGE, format!("bytes {start}-{last}/{size}"))
    } else {
        response.status(StatusCode::OK)
    };
    response.body(body).unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
