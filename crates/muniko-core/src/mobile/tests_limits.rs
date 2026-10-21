//! Oversized JSON bodies are refused before they are parsed.

use reqwest::StatusCode;

use super::tests::Fixture;
use crate::constants::mobile::ROUTE_SESSION;
use crate::constants::server::MAX_SMALL_JSON_BYTES;
use crate::lifecycle::Lifecycle;

#[tokio::test]
async fn redeem_refuses_an_oversized_body() {
    let mut fx = Fixture::start().await;
    let body = format!(r#"{{"token":"{}"}}"#, "x".repeat(MAX_SMALL_JSON_BYTES));
    let response = fx.client.post(fx.url(ROUTE_SESSION)).header("content-type", "application/json").body(body).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    fx.server.stop().await.unwrap();
}
