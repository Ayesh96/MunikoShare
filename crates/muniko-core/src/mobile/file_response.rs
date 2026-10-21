//! Streams a file to the phone with Range support.

use std::path::Path;

use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use super::range::{parse_range, RangeRequest};
use crate::constants::io::READ_BUFFER_BYTES;
use crate::constants::mobile::BYTES_UNIT;
use crate::messages::mobile as msg;

/// Streams a file with Range support and an attachment name.
/// `hold` is kept alive until the body is fully streamed or dropped (e.g. a cache lease).
pub async fn serve_file(path: &Path, name: &str, range: Option<&str>, hold: impl Send + 'static) -> Result<Response, StatusCode> {
    let mut file = File::open(path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let size = file.metadata().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.len();
    let (status, start, length, content_range) = match parse_range(range, size) {
        RangeRequest::Whole => (StatusCode::OK, 0, size, None),
        RangeRequest::Part { start, end } => (StatusCode::PARTIAL_CONTENT, start, end - start + 1, Some(format!("{BYTES_UNIT} {start}-{end}/{size}"))),
        RangeRequest::Unsatisfiable => return Ok(unsatisfiable(size)),
    };
    file.seek(std::io::SeekFrom::Start(start)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = Body::from_stream(ReaderStream::with_capacity(file.take(length), READ_BUFFER_BYTES).map(move |chunk| {
        let _held = &hold;
        chunk
    }));
    let mut response = (status, body).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_LENGTH, length.into());
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static(BYTES_UNIT));
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    if let Ok(value) = HeaderValue::from_str(&content_disposition(name)) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    if let Some(value) = content_range.and_then(|text| HeaderValue::from_str(&text).ok()) {
        headers.insert(header::CONTENT_RANGE, value);
    }
    Ok(response)
}

fn unsatisfiable(size: u64) -> Response {
    let mut response = (StatusCode::RANGE_NOT_SATISFIABLE, msg::RANGE_NOT_SATISFIABLE).into_response();
    if let Ok(value) = HeaderValue::from_str(&format!("{BYTES_UNIT} */{size}")) {
        response.headers_mut().insert(header::CONTENT_RANGE, value);
    }
    response
}

// An ASCII fallback for old browsers plus the exact UTF-8 name.
fn content_disposition(name: &str) -> String {
    let fallback: String = name.chars().map(|c| if c.is_ascii_graphic() && c != '"' && c != '\\' || c == ' ' { c } else { '_' }).collect();
    format!("attachment; filename=\"{fallback}\"; filename*=UTF-8''{}", utf8_percent_encode(name, NON_ALPHANUMERIC))
}
