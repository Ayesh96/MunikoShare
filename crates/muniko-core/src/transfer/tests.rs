//! End-to-end tests over a real TLS socket on loopback.

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use sha2::{Digest, Sha256};
use tempfile::TempDir;

use super::{OfferResponder, TransferClient, TransferRoutes};
use crate::constants::transfer::*;
use crate::device::DeviceInfo;
use crate::events::{EventBus, NodeEvent, TransferDirection};
use crate::identity::Identity;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::net::pinned_client_config;
use crate::server::{HttpServer, RouteProvider};
use crate::trust::{MemoryTrustStore, TrustStore, TrustedDevice};
use crate::util::fingerprint::fingerprint;
use crate::constants::testing::LOOPBACK;
use crate::util::test_support::{shared_test_identity, test_identity};
use crate::web::WebRoutes;
use crate::MunikoError;


pub(super) struct Fixture {
    pub(super) server: HttpServer,
    pub(super) server_identity: Arc<Identity>,
    pub(super) client: TransferClient,
    pub(super) client_identity: Arc<Identity>,
    pub(super) bus: EventBus,
    pub(super) events: tokio::sync::broadcast::Receiver<NodeEvent>,
    pub(super) responder: OfferResponder,
    pub(super) trust: Arc<MemoryTrustStore>,
    pub(super) download_dir: TempDir,
    pub(super) source_dir: TempDir,
}

impl Fixture {
    pub(super) async fn start() -> Self {
        Self::start_with(true, OFFER_TIMEOUT, TRANSFER_IDLE_TIMEOUT).await
    }

    pub(super) async fn start_with(trust_client: bool, offer_timeout: Duration, idle_timeout: Duration) -> Self {
        let server_identity = shared_test_identity();
        let client_identity = shared_test_identity();
        let trust = Arc::new(MemoryTrustStore::new());
        if trust_client {
            let device = TrustedDevice { id: client_identity.device_id().to_string(), name: "c".into(), os: "test".into(), paired_at: 0 };
            trust.add(device).unwrap();
        }
        let bus = EventBus::new();
        let events = bus.subscribe();
        let download_dir = TempDir::new().unwrap();
        let routes = TransferRoutes::with_timeouts(download_dir.path().to_path_buf(), bus.clone(), trust.clone(), offer_timeout, idle_timeout);
        let responder = routes.responder();
        let providers: Vec<Arc<dyn RouteProvider>> = vec![Arc::new(routes), Arc::new(WebRoutes::new())];
        let mut server = HttpServer::new(server_identity.clone(), providers);
        server.start().await.unwrap();
        let client = TransferClient::new(client_identity.clone(), bus.clone());
        Self { server, server_identity, client, client_identity, bus, events, responder, trust, download_dir, source_dir: TempDir::new().unwrap() }
    }

    pub(super) fn peer_with_id(&self, id: &str) -> DeviceInfo {
        DeviceInfo {
            id: id.to_string(),
            name: "peer".into(),
            os: "test".into(),
            addresses: vec![LOOPBACK.parse::<IpAddr>().unwrap()],
            port: self.server.port().unwrap(),
            protocol_version: 1,
        }
    }

    pub(super) fn peer(&self) -> DeviceInfo {
        self.peer_with_id(self.server_identity.device_id())
    }

    pub(super) fn source_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.source_dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    pub(super) fn http_client_as(&self, identity: &Identity) -> reqwest::Client {
        let config = pinned_client_config(self.server_identity.device_id(), Some(identity)).unwrap();
        reqwest::Client::builder().tls_backend_preconfigured(config).build().unwrap()
    }

    pub(super) fn pinned_http_client(&self) -> reqwest::Client {
        self.http_client_as(&self.client_identity)
    }

    pub(super) fn url(&self, path: &str) -> String {
        format!("https://{LOOPBACK}:{}{path}", self.server.port().unwrap())
    }

    // Answers the next incoming offer in the background, like a user clicking a button.
    pub(super) fn answer_next_offer(&self, accept: bool) {
        let mut events = self.bus.subscribe();
        let responder = self.responder.clone();
        tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                if let NodeEvent::IncomingOffer { offer_id, .. } = event {
                    responder.respond_to_offer(&offer_id, accept).unwrap();
                    break;
                }
            }
        });
    }

    // Bypasses TransferClient so tests can send names and hashes it would never produce.
    pub(super) async fn raw_offer(&self, client: &reqwest::Client, name: &str, content: &[u8], sha256: &str) -> reqwest::Response {
        let manifest = serde_json::json!({ "files": [{ "path": name, "size": content.len(), "sha256": sha256 }] });
        client.post(self.url(ROUTE_TRANSFERS)).json(&manifest).send().await.unwrap()
    }

    pub(super) async fn raw_put(&self, client: &reqwest::Client, transfer_id: &str, index: usize, content: &[u8]) -> reqwest::Response {
        let url = self.url(&format!("{ROUTE_TRANSFERS}/{transfer_id}/files/{index}"));
        client.put(url).body(content.to_vec()).send().await.unwrap()
    }

    // Offers one file as the trusted client, accepts it, and returns the transfer id.
    pub(super) async fn accepted_transfer(&self, name: &str, content: &[u8]) -> String {
        self.answer_next_offer(true);
        let response = self.raw_offer(&self.pinned_http_client(), name, content, &sha_hex(content)).await;
        assert_eq!(response.status(), 200);
        response.json::<serde_json::Value>().await.unwrap()["transfer_id"].as_str().unwrap().to_string()
    }

    pub(super) fn downloaded_names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.download_dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    pub(super) fn drain_events(&mut self) -> Vec<NodeEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            events.push(event);
        }
        events
    }
}

pub(super) fn sha_hex(content: &[u8]) -> String {
    hex::encode(Sha256::digest(content))
}

#[tokio::test]
async fn accepted_offer_delivers_files_intact_with_events() {
    let mut fx = Fixture::start().await;
    let big: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
    let paths = vec![fx.source_file("big.bin", &big), fx.source_file("small.txt", b"hello")];
    let total = big.len() as u64 + 5;
    fx.answer_next_offer(true);

    fx.client.send_files(&fx.peer(), &paths).await.unwrap();

    assert_eq!(std::fs::read(fx.download_dir.path().join("big.bin")).unwrap(), big);
    assert_eq!(std::fs::read(fx.download_dir.path().join("small.txt")).unwrap(), b"hello");
    assert_eq!(fx.downloaded_names(), vec!["big.bin".to_string(), "small.txt".to_string()]);
    let events = fx.drain_events();
    let offer = events.iter().find_map(|e| match e {
        NodeEvent::IncomingOffer { from_device_id, files, total_size, .. } => Some((from_device_id.clone(), files.len(), *total_size)),
        _ => None,
    });
    assert_eq!(offer, Some((fx.client_identity.device_id().to_string(), 2, total)));
    assert!(events.iter().any(|e| matches!(e, NodeEvent::OfferClosed { accepted: true, .. })));
    assert_eq!(events.iter().filter(|e| matches!(e, NodeEvent::FileReceived { .. })).count(), 2);
    assert_eq!(events.iter().filter(|e| matches!(e, NodeEvent::FileSent { .. })).count(), 2);
    for direction in [TransferDirection::Incoming, TransferDirection::Outgoing] {
        let last = events.iter().rev().find_map(|e| match e {
            NodeEvent::TransferProgress { direction: d, bytes_done, bytes_total, .. } if *d == direction => Some((*bytes_done, *bytes_total)),
            _ => None,
        });
        assert_eq!(last, Some((total, total)), "{direction:?}");
    }
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn declined_offer_fails_the_sender_and_closes_the_offer() {
    let mut fx = Fixture::start().await;
    let source = fx.source_file("a.txt", b"hello");
    fx.answer_next_offer(false);

    let error = fx.client.send_files(&fx.peer(), &[source]).await.unwrap_err();

    assert!(error.to_string().contains(crate::messages::transfer::PEER_DECLINED), "{error}");
    assert!(fx.downloaded_names().is_empty());
    let events = fx.drain_events();
    assert!(events.iter().any(|e| matches!(e, NodeEvent::OfferClosed { accepted: false, .. })));
    assert!(events.iter().any(|e| matches!(e, NodeEvent::TransferFailed { .. })));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn unanswered_offer_times_out_with_408() {
    let mut fx = Fixture::start_with(true, Duration::from_millis(200), TRANSFER_IDLE_TIMEOUT).await;
    let response = fx.raw_offer(&fx.pinned_http_client(), "a.txt", b"x", &sha_hex(b"x")).await;
    assert_eq!(response.status(), 408);
    let events = fx.drain_events();
    assert!(events.iter().any(|e| matches!(e, NodeEvent::OfferClosed { accepted: false, .. })));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn responding_to_a_closed_offer_is_an_error() {
    let fx = Fixture::start().await;
    assert!(fx.responder.respond_to_offer("nope", true).is_err());
}

#[tokio::test]
async fn untrusted_sender_gets_403_and_no_prompt() {
    let mut fx = Fixture::start_with(false, OFFER_TIMEOUT, TRANSFER_IDLE_TIMEOUT).await;
    let response = fx.raw_offer(&fx.pinned_http_client(), "a.txt", b"x", &sha_hex(b"x")).await;
    assert_eq!(response.status(), 403);
    assert!(fx.drain_events().iter().all(|e| !matches!(e, NodeEvent::IncomingOffer { .. })));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn upload_from_another_device_is_forbidden() {
    let mut fx = Fixture::start().await;
    let transfer_id = fx.accepted_transfer("a.txt", b"data").await;
    let other = fx.http_client_as(&test_identity());
    assert_eq!(fx.raw_put(&other, &transfer_id, 0, b"data").await.status(), 403);
    assert!(fx.downloaded_names().is_empty());
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn unknown_transfer_and_file_index_get_404() {
    let mut fx = Fixture::start().await;
    let client = fx.pinned_http_client();
    assert_eq!(fx.raw_put(&client, "missing", 0, b"x").await.status(), 404);
    let transfer_id = fx.accepted_transfer("a.txt", b"data").await;
    assert_eq!(fx.raw_put(&client, &transfer_id, 5, b"data").await.status(), 404);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn idle_accepted_transfer_expires() {
    let mut fx = Fixture::start_with(true, OFFER_TIMEOUT, Duration::from_millis(100)).await;
    let transfer_id = fx.accepted_transfer("a.txt", b"data").await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(fx.raw_put(&fx.pinned_http_client(), &transfer_id, 0, b"data").await.status(), 404);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn a_file_cannot_be_uploaded_twice() {
    let mut fx = Fixture::start().await;
    let client = fx.pinned_http_client();
    fx.answer_next_offer(true);
    let manifest = serde_json::json!({ "files": [
        { "path": "a.txt", "size": 1, "sha256": sha_hex(b"a") },
        { "path": "b.txt", "size": 1, "sha256": sha_hex(b"b") },
    ] });
    let response = client.post(fx.url(ROUTE_TRANSFERS)).json(&manifest).send().await.unwrap();
    let transfer_id = response.json::<serde_json::Value>().await.unwrap()["transfer_id"].as_str().unwrap().to_string();
    assert_eq!(fx.raw_put(&client, &transfer_id, 0, b"a").await.status(), 201);
    assert_eq!(fx.raw_put(&client, &transfer_id, 0, b"a").await.status(), 409);
    assert_eq!(fx.raw_put(&client, &transfer_id, 1, b"b").await.status(), 201);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn invalid_manifests_are_rejected() {
    let mut fx = Fixture::start().await;
    let client = fx.pinned_http_client();
    let post = |body: serde_json::Value| client.post(fx.url(ROUTE_TRANSFERS)).json(&body).send();
    assert_eq!(post(serde_json::json!({ "files": [] })).await.unwrap().status(), 400);
    let bad_hash = serde_json::json!({ "files": [{ "path": "a", "size": 1, "sha256": "zz" }] });
    assert_eq!(post(bad_hash).await.unwrap().status(), 400);
    let bad_name = serde_json::json!({ "files": [{ "path": "..", "size": 1, "sha256": "0".repeat(64) }] });
    assert_eq!(post(bad_name).await.unwrap().status(), 400);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn empty_folders_are_rejected_before_offering() {
    let mut fx = Fixture::start().await;
    let error = fx.client.send_files(&fx.peer(), &[fx.source_dir.path().to_path_buf()]).await.unwrap_err();
    assert!(matches!(error, MunikoError::Transfer(_)));
    assert!(fx.drain_events().iter().all(|e| !matches!(e, NodeEvent::IncomingOffer { .. })));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn rejects_wrong_peer_id() {
    let mut fx = Fixture::start().await;
    let source = fx.source_file("a.txt", b"hello");
    let peer = fx.peer_with_id(&fingerprint(b"someone else"));

    let error = fx.client.send_files(&peer, &[source]).await.unwrap_err();

    assert!(matches!(error, MunikoError::Transfer(_)));
    assert!(fx.downloaded_names().is_empty());
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn duplicate_names_get_numbered() {
    let mut fx = Fixture::start().await;
    let source = fx.source_file("report.txt", b"one");
    for _ in 0..2 {
        fx.answer_next_offer(true);
        fx.client.send_files(&fx.peer(), &[source.clone()]).await.unwrap();
    }
    assert_eq!(fx.downloaded_names(), vec!["report (1).txt".to_string(), "report.txt".to_string()]);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn bad_checksum_is_rejected_and_cleaned_up() {
    let mut fx = Fixture::start().await;
    fx.answer_next_offer(true);
    let client = fx.pinned_http_client();
    let response = fx.raw_offer(&client, "x.txt", b"data", &"0".repeat(64)).await;
    let transfer_id = response.json::<serde_json::Value>().await.unwrap()["transfer_id"].as_str().unwrap().to_string();
    assert_eq!(fx.raw_put(&client, &transfer_id, 0, b"data").await.status(), 422);
    assert!(fx.downloaded_names().is_empty());
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn authenticated_non_api_requests_get_404() {
    let mut fx = Fixture::start().await;
    let client = fx.pinned_http_client();
    assert_eq!(client.get(fx.url("/")).send().await.unwrap().status(), 404);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn lifecycle_rules() {
    let mut fx = Fixture::start().await;
    assert_eq!(fx.server.state(), LifecycleState::Running);
    assert!(matches!(fx.server.start().await, Err(MunikoError::InvalidState(_))));
    fx.server.stop().await.unwrap();
    assert_eq!(fx.server.state(), LifecycleState::Stopped);
    assert_eq!(fx.server.port(), None);
}
