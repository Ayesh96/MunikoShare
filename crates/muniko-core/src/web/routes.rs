use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;

use crate::constants::web::{API_PREFIX, RELEASES_PLACEHOLDER, RELEASES_URL, ROUTE_DOWNLOAD};
use crate::messages::web as msg;
use crate::server::{RequestPeer, RouteProvider};

const DOWNLOAD_PAGE: &str = include_str!("download.html");

/// Serves the download page and sends browsers there; unknown API paths stay 404.
pub struct WebRoutes {
    page: Arc<str>,
}

impl WebRoutes {
    pub fn new() -> Self {
        Self { page: Arc::from(DOWNLOAD_PAGE.replace(RELEASES_PLACEHOLDER, RELEASES_URL)) }
    }
}

impl Default for WebRoutes {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteProvider for WebRoutes {
    fn routes(&self) -> Router {
        Router::new().route(ROUTE_DOWNLOAD, get(download_page)).fallback(fallback).with_state(self.page.clone())
    }
}

async fn download_page(State(page): State<Arc<str>>) -> Html<String> {
    Html(page.to_string())
}

// Only certificate-less clients (browsers) are redirected; apps and API paths get a plain 404.
async fn fallback(Extension(peer): Extension<RequestPeer>, uri: Uri) -> Response {
    if peer.device_id.is_none() && !uri.path().starts_with(API_PREFIX) {
        tracing::debug!("{}", msg::BROWSER_REDIRECTED);
        return Redirect::to(ROUTE_DOWNLOAD).into_response();
    }
    (StatusCode::NOT_FOUND, msg::NOT_FOUND).into_response()
}
