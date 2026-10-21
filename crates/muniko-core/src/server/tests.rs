//! The server over a real TLS socket: client certs, redirects and the download page.

use std::sync::Arc;

use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
use tempfile::TempDir;

use super::{HttpServer, RouteProvider};
use crate::constants::transfer::ROUTE_TRANSFERS;
use crate::constants::web::ROUTE_DOWNLOAD;
use crate::events::EventBus;
use crate::identity::Identity;
use crate::lifecycle::Lifecycle;
use crate::net::pinned_client_config;
use crate::transfer::TransferRoutes;
use crate::trust::MemoryTrustStore;
use crate::constants::testing::LOOPBACK;
use crate::util::test_support::{shared_test_identity, test_identity};
use crate::web::WebRoutes;


struct Fixture {
    server: HttpServer,
    server_id: String,
    _downloads: TempDir,
}

impl Fixture {
    async fn start() -> Self {
        let identity = shared_test_identity();
        let downloads = TempDir::new().unwrap();
        let providers: Vec<Arc<dyn RouteProvider>> =
            vec![Arc::new(TransferRoutes::new(downloads.path().to_path_buf(), EventBus::new(), Arc::new(MemoryTrustStore::new()))), Arc::new(WebRoutes::new())];
        let mut server = HttpServer::new(identity.clone(), providers);
        server.start().await.unwrap();
        Self { server, server_id: identity.device_id().to_string(), _downloads: downloads }
    }

    fn url(&self, path: &str) -> String {
        format!("https://{LOOPBACK}:{}{path}", self.server.port().unwrap())
    }

    // Mimics a browser (no cert) or an app (with cert); redirects are left for the test to see.
    fn client(&self, identity: Option<&Identity>) -> Client {
        let config = pinned_client_config(&self.server_id, identity).unwrap();
        Client::builder().tls_backend_preconfigured(config).redirect(Policy::none()).build().unwrap()
    }
}

#[tokio::test]
async fn upload_without_client_cert_is_unauthorized() {
    let mut fx = Fixture::start().await;
    let response = fx.client(None).post(fx.url(ROUTE_TRANSFERS)).body("x").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn upload_with_client_cert_passes_authentication() {
    let mut fx = Fixture::start().await;
    let me = test_identity();
    let response = fx
        .client(Some(&me))
        .post(fx.url(ROUTE_TRANSFERS))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(r#"{"files":[]}"#)
        .send()
        .await
        .unwrap();
    // Authenticated (not 401), then refused because the device is not paired.
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn browser_root_request_redirects_to_download_page() {
    let mut fx = Fixture::start().await;
    let response = fx.client(None).get(fx.url("/some/page")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()["location"], ROUTE_DOWNLOAD);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn download_page_is_served_as_html() {
    let mut fx = Fixture::start().await;
    let response = fx.client(None).get(fx.url(ROUTE_DOWNLOAD)).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers()["content-type"].to_str().unwrap().starts_with("text/html"));
    let body = response.text().await.unwrap();
    assert!(body.contains("Muniko Share"));
    assert!(!body.contains("{{"));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn unknown_api_route_is_not_found() {
    let mut fx = Fixture::start().await;
    let response = fx.client(None).get(fx.url("/api/v1/nothing")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn app_clients_do_not_get_redirected() {
    let mut fx = Fixture::start().await;
    let me = test_identity();
    let response = fx.client(Some(&me)).get(fx.url("/")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    fx.server.stop().await.unwrap();
}
