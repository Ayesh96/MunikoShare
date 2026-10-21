use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast::Receiver;

use super::*;
use crate::constants::pairing::{CODE_LENGTH, ROUTE_REQUEST};
use crate::device::DeviceInfo;
use crate::events::{EventBus, NodeEvent};
use crate::identity::Identity;
use crate::lifecycle::Lifecycle;
use crate::server::{HttpServer, RouteProvider};
use crate::trust::{MemoryTrustStore, TrustStore};
use crate::constants::testing::EVENT_WAIT;
use crate::util::test_support::shared_test_identity;


fn profile(name: &str) -> PairingProfile {
    PairingProfile { name: name.into(), os: "test".into() }
}

#[test]
fn code_has_fixed_digit_length() {
    for _ in 0..200 {
        let code = generate_code();
        assert_eq!(code.len(), CODE_LENGTH as usize);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}

#[test]
fn correct_code_is_accepted_once() {
    let book = PendingBook::new(Duration::from_secs(60), 5);
    let code = book.begin("a", profile("A"));
    assert_eq!(book.verify("a", &code), Verdict::Accepted(profile("A")));
    assert_eq!(book.verify("a", &code), Verdict::NoPending);
}

#[test]
fn code_is_bound_to_device_id() {
    let book = PendingBook::new(Duration::from_secs(60), 5);
    let code = book.begin("a", profile("A"));
    assert_eq!(book.verify("b", &code), Verdict::NoPending);
}

#[test]
fn expired_code_is_rejected_and_removed() {
    let book = PendingBook::new(Duration::from_millis(20), 5);
    let code = book.begin("a", profile("A"));
    std::thread::sleep(Duration::from_millis(40));
    assert_eq!(book.verify("a", &code), Verdict::Expired);
    assert_eq!(book.verify("a", &code), Verdict::NoPending);
}

#[test]
fn attempts_are_limited() {
    let book = PendingBook::new(Duration::from_secs(60), 3);
    let code = book.begin("a", profile("A"));
    let wrong = if code == "000000" { "000001" } else { "000000" };
    assert_eq!(book.verify("a", wrong), Verdict::WrongCode);
    assert_eq!(book.verify("a", wrong), Verdict::WrongCode);
    assert_eq!(book.verify("a", wrong), Verdict::Expired);
    assert_eq!(book.verify("a", &code), Verdict::NoPending);
}

struct Node {
    identity: Arc<Identity>,
    store: Arc<MemoryTrustStore>,
    service: PairingService,
    events: Receiver<NodeEvent>,
    server: HttpServer,
}

async fn node(name: &str, max_attempts: u32) -> Node {
    let identity = shared_test_identity();
    let store = Arc::new(MemoryTrustStore::new());
    let bus = EventBus::new();
    let events = bus.subscribe();
    let service = PairingService::with_limits(identity.clone(), store.clone(), bus, profile(name), Duration::from_secs(60), max_attempts);
    let providers: Vec<Arc<dyn RouteProvider>> = vec![Arc::new(service.routes())];
    let mut server = HttpServer::new(identity.clone(), providers);
    server.start().await.unwrap();
    Node { identity, store, service, events, server }
}

impl Node {
    fn info(&self) -> DeviceInfo {
        DeviceInfo {
            id: self.identity.device_id().to_string(),
            name: "peer".into(),
            os: "test".into(),
            addresses: vec!["127.0.0.1".parse::<IpAddr>().unwrap()],
            port: self.server.port().unwrap(),
            protocol_version: 1,
        }
    }
}

async fn next_event(events: &mut Receiver<NodeEvent>) -> NodeEvent {
    tokio::time::timeout(EVENT_WAIT, events.recv()).await.unwrap().unwrap()
}

async fn shown_code(events: &mut Receiver<NodeEvent>) -> String {
    match next_event(events).await {
        NodeEvent::PairingRequested { code, .. } => code,
        other => panic!("unexpected event {other:?}"),
    }
}

#[tokio::test]
async fn full_pairing_flow_trusts_both_sides() {
    let mut a = node("A", 5).await;
    let mut b = node("B", 5).await;
    a.service.request(&b.info()).await.unwrap();
    let code = shown_code(&mut b.events).await;
    a.service.confirm(&b.info(), &code).await.unwrap();

    assert!(a.store.is_trusted(b.identity.device_id()));
    assert!(b.store.is_trusted(a.identity.device_id()));
    assert_eq!(a.service.trusted()[0].name, "B");
    assert_eq!(b.service.trusted()[0].name, "A");
    assert!(matches!(next_event(&mut a.events).await, NodeEvent::PairingCompleted { device_name, .. } if device_name == "B"));
    assert!(matches!(next_event(&mut b.events).await, NodeEvent::PairingCompleted { device_name, .. } if device_name == "A"));

    a.service.unpair(b.identity.device_id()).unwrap();
    assert!(!a.store.is_trusted(b.identity.device_id()));
    assert!(matches!(next_event(&mut a.events).await, NodeEvent::DeviceUnpaired { .. }));
}

#[tokio::test]
async fn wrong_code_is_forbidden_and_publishes_failure() {
    let a = node("A", 5).await;
    let mut b = node("B", 5).await;
    let mut a_events = a.service_events();
    a.service.request(&b.info()).await.unwrap();
    let code = shown_code(&mut b.events).await;
    let wrong = if code == "000000" { "000001" } else { "000000" };
    let error = a.service.confirm(&b.info(), wrong).await.unwrap_err().to_string();
    assert!(error.contains("403"), "{error}");
    assert!(matches!(next_event(&mut a_events).await, NodeEvent::PairingFailed { .. }));
    assert!(!b.store.is_trusted(a.identity.device_id()));
}

#[tokio::test]
async fn too_many_wrong_codes_gone() {
    let a = node("A", 2).await;
    let mut b = node("B", 2).await;
    a.service.request(&b.info()).await.unwrap();
    let code = shown_code(&mut b.events).await;
    let wrong = if code == "000000" { "000001" } else { "000000" };
    assert!(a.service.confirm(&b.info(), wrong).await.unwrap_err().to_string().contains("403"));
    assert!(a.service.confirm(&b.info(), wrong).await.unwrap_err().to_string().contains("410"));
    assert!(matches!(next_event(&mut b.events).await, NodeEvent::PairingFailed { .. }));
}

#[tokio::test]
async fn unauthenticated_request_is_unauthorized() {
    let b = node("B", 5).await;
    let config = crate::net::pinned_client_config(b.identity.device_id(), None).unwrap();
    let http = reqwest::Client::builder().tls_backend_preconfigured(config).build().unwrap();
    let url = format!("https://127.0.0.1:{}{ROUTE_REQUEST}", b.server.port().unwrap());
    let response = http.post(url).json(&profile("X")).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

impl Node {
    // Subscribing after construction would miss nothing since no events were published yet.
    fn service_events(&self) -> Receiver<NodeEvent> {
        self.events.resubscribe()
    }
}
