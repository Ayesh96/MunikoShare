//! Serves the embedded mobile app with a fallback to its index page.

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use rust_embed::RustEmbed;

use super::service::MobileService;
use crate::constants::mobile::{CACHE_ASSETS, CACHE_NO_CACHE, INDEX_FILE, PAGE_NOT_BUILT, ROUTE_MOBILE_DIR};
use crate::constants::web::ROUTE_DOWNLOAD;
use crate::messages::mobile as msg;

// allow_missing keeps the core compiling before the mobile app has been built.
#[derive(RustEmbed)]
#[folder = "../../apps/mobile-web/dist"]
#[allow_missing = true]
struct MobileAssets;

pub async fn root(State(service): State<MobileService>) -> Response {
    page(&service, "")
}

pub async fn asset(State(service): State<MobileService>, Path(path): Path<String>) -> Response {
    page(&service, &path)
}

pub async fn trailing_slash() -> Redirect {
    Redirect::permanent(ROUTE_MOBILE_DIR)
}

// A disabled feature looks like any unknown path to a browser.
fn page(service: &MobileService, path: &str) -> Response {
    if !service.is_enabled() {
        return Redirect::to(ROUTE_DOWNLOAD).into_response();
    }
    let name = if path.is_empty() { INDEX_FILE } else { path };
    match MobileAssets::get(name) {
        Some(file) => file_response(name, &file.metadata.mimetype().to_string(), file.data.into_owned()),
        None if is_page_route(name) => index_response(),
        None => (StatusCode::NOT_FOUND, msg::NOT_FOUND).into_response(),
    }
}

// Paths without a file extension are client-side routes of the single-page app.
fn is_page_route(name: &str) -> bool {
    !name.rsplit('/').next().unwrap_or_default().contains('.')
}

fn index_response() -> Response {
    match MobileAssets::get(INDEX_FILE) {
        Some(file) => file_response(INDEX_FILE, &file.metadata.mimetype().to_string(), file.data.into_owned()),
        None => ([(header::CACHE_CONTROL, CACHE_NO_CACHE)], Html(PAGE_NOT_BUILT)).into_response(),
    }
}

fn file_response(name: &str, content_type: &str, data: Vec<u8>) -> Response {
    let cache = if name == INDEX_FILE { CACHE_NO_CACHE } else { CACHE_ASSETS };
    ([(header::CONTENT_TYPE, content_type.to_string()), (header::CACHE_CONTROL, cache.to_string())], data).into_response()
}
