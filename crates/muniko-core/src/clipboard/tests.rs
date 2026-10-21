//! End-to-end tests over a real TLS socket on loopback with a fake clipboard.

use std::net::IpAddr;
use std::sync::Arc;

use super::tests_support::{test_image, FakeClipboard};
use super::{ClipboardContent, ClipboardRoutes, ClipboardService};
use crate::constants::clipboard::{JSON_CONTENT_TYPE, MAX_CLIPBOARD_BYTES, ROUTE_CLIPBOARD};
use crate::device::DeviceInfo;
use crate::events::{EventBus, NodeEvent};
use crate::identity::Identity;
use crate::lifecycle::Lifecycle;
use crate::net::pinned_client_config;
use crate::server::{HttpServer, RouteProvider};
use crate::trust::{MemoryTrustStore, TrustStore, TrustedDevice};
use crate::constants::testing::{EVENT_WAIT, LOOPBACK};
use crate::util::test_support::shared_test_identity;


struct Fixture {
    server: HttpServer,
    server_identity: Arc<Identity>,
    sender_identity: Arc<Identity>,
    sender: ClipboardService,
    sender_clipboard: Arc<FakeClipboard>,
    receiver_clipboard: Arc<FakeClipboard>,
    trust: Arc<MemoryTrustStore>,
    events: tokio::sync::broadcast::Receiver<NodeEvent>,
}

impl Fixture {
    async fn start(trusted: bool) -> Self {
        let server_identity = shared_test_identity();
        let sender_identity = shared_test_identity();
        let trust = Arc::new(MemoryTrustStore::new());
        if trusted {
            trust
                .add(TrustedDevice { id: sender_identity.device_id().to_string(), name: "sender".into(), os: "test".into(), paired_at: 0 })
                .unwrap();
        }
        let bus = EventBus::new();
        let events = bus.subscribe();
        let receiver_clipboard = FakeClipboard::empty();
        let routes = ClipboardRoutes::new(receiver_clipboard.clone(), trust.clone(), bus);
        let providers: Vec<Arc<dyn RouteProvider>> = vec![Arc::new(routes)];
        let mut server = HttpServer::new(server_identity.clone(), providers);
        server.start().await.unwrap();
        let sender_clipboard = FakeClipboard::with(ClipboardContent::Text("hello from sender".into()));
        let sender = ClipboardService::new(sender_identity.clone(), sender_clipboard.clone());
        Self { server, server_identity, sender_identity, sender, sender_clipboard, receiver_clipboard, trust, events }
    }

    fn peer(&self) -> DeviceInfo {
        DeviceInfo {
            id: self.server_identity.device_id().to_string(),
            name: "peer".into(),
            os: "test".into(),
            addresses: vec![LOOPBACK.parse::<IpAddr>().unwrap()],
            port: self.server.port().unwrap(),
            protocol_version: 1,
        }
    }

    fn url(&self) -> String {
        format!("https://{LOOPBACK}:{}{ROUTE_CLIPBOARD}", self.server.port().unwrap())
    }

    // Bypasses the client so tests can send what it would refuse to.
    async fn raw_post(&self, text: &str) -> reqwest::Response {
        let config = pinned_client_config(self.server_identity.device_id(), Some(&self.sender_identity)).unwrap();
        let http = reqwest::Client::builder().tls_backend_preconfigured(config).build().unwrap();
        http.post(self.url()).header("content-type", JSON_CONTENT_TYPE).body(serde_json::json!({ "text": text }).to_string()).send().await.unwrap()
    }
}

#[tokio::test]
async fn trusted_push_writes_clipboard_and_publishes_event() {
    let mut fx = Fixture::start(true).await;
    fx.sender.push_text(&fx.peer(), "héllo\nworld").await.unwrap();

    assert_eq!(fx.receiver_clipboard.text().as_deref(), Some("héllo\nworld"));
    let event = tokio::time::timeout(EVENT_WAIT, fx.events.recv()).await.unwrap().unwrap();
    match event {
        NodeEvent::ClipboardReceived { from_device_id, text } => {
            assert_eq!(from_device_id, fx.sender_identity.device_id());
            assert_eq!(text, "héllo\nworld");
        }
        other => panic!("unexpected event {other:?}"),
    }
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn push_current_sends_local_clipboard() {
    let mut fx = Fixture::start(true).await;
    fx.sender.push_current(&fx.peer()).await.unwrap();
    assert_eq!(fx.receiver_clipboard.text().as_deref(), Some("hello from sender"));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn push_current_fails_when_local_clipboard_empty() {
    let mut fx = Fixture::start(true).await;
    fx.sender_clipboard.clear();
    let error = fx.sender.push_current(&fx.peer()).await.unwrap_err();
    assert!(error.to_string().contains(crate::messages::clipboard::LOCAL_CLIPBOARD_EMPTY));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn untrusted_push_is_forbidden() {
    let mut fx = Fixture::start(false).await;
    assert_eq!(fx.raw_post("secret").await.status(), 403);
    assert!(fx.sender.push_text(&fx.peer(), "secret").await.is_err());
    assert_eq!(fx.receiver_clipboard.current(), None);
    assert!(!fx.trust.is_trusted(fx.sender_identity.device_id()));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn oversized_text_is_rejected() {
    let mut fx = Fixture::start(true).await;
    let big = "a".repeat(MAX_CLIPBOARD_BYTES + 1);
    assert_eq!(fx.raw_post(&big).await.status(), 413);
    assert!(fx.sender.push_text(&fx.peer(), &big).await.is_err());
    assert_eq!(fx.receiver_clipboard.current(), None);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn empty_text_is_rejected() {
    let mut fx = Fixture::start(true).await;
    assert_eq!(fx.raw_post("").await.status(), 400);
    assert!(fx.sender.push_text(&fx.peer(), "").await.is_err());
    assert_eq!(fx.receiver_clipboard.current(), None);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn missing_client_certificate_is_unauthorized() {
    let mut fx = Fixture::start(true).await;
    let config = pinned_client_config(fx.server_identity.device_id(), None).unwrap();
    let http = reqwest::Client::builder().tls_backend_preconfigured(config).build().unwrap();
    let response = http.post(fx.url()).header("content-type", JSON_CONTENT_TYPE).body(r#"{"text":"hi"}"#).send().await.unwrap();
    assert_eq!(response.status(), 401);
    assert_eq!(fx.receiver_clipboard.current(), None);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn image_push_round_trips_with_event() {
    let mut fx = Fixture::start(true).await;
    let image = test_image(5, 4);
    fx.sender.push_content(&fx.peer(), &image).await.unwrap();
    assert_eq!(fx.receiver_clipboard.current(), Some(image));
    match tokio::time::timeout(EVENT_WAIT, fx.events.recv()).await.unwrap().unwrap() {
        NodeEvent::ClipboardImageReceived { width: 5, height: 4, .. } => {}
        other => panic!("unexpected event {other:?}"),
    }
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn push_current_sends_image() {
    let mut fx = Fixture::start(true).await;
    let image = test_image(3, 3);
    fx.sender_clipboard.user_copy(image.clone());
    fx.sender.push_current(&fx.peer()).await.unwrap();
    assert_eq!(fx.receiver_clipboard.current(), Some(image));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn untrusted_image_push_is_forbidden() {
    let mut fx = Fixture::start(false).await;
    assert!(fx.sender.push_content(&fx.peer(), &test_image(2, 2)).await.is_err());
    assert_eq!(fx.receiver_clipboard.current(), None);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn oversized_image_is_refused_before_sending() {
    let mut fx = Fixture::start(true).await;
    let huge = ClipboardContent::Image { width: 1, height: 1, png: vec![0; crate::constants::clipboard::MAX_IMAGE_PNG_BYTES + 1] };
    assert!(fx.sender.push_content(&fx.peer(), &huge).await.is_err());
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn wrong_origin_hash_is_rejected() {
    let mut fx = Fixture::start(true).await;
    let config = pinned_client_config(fx.server_identity.device_id(), Some(&fx.sender_identity)).unwrap();
    let http = reqwest::Client::builder().tls_backend_preconfigured(config).build().unwrap();
    let body = serde_json::json!({ "kind": "text", "text": "hi", "origin_hash": "deadbeef" }).to_string();
    let response = http.post(fx.url()).header("content-type", JSON_CONTENT_TYPE).body(body).send().await.unwrap();
    assert_eq!(response.status(), 400);
    fx.server.stop().await.unwrap();
}
