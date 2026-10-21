//! The phone page and API over a real loopback server, with a client that has no certificate.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{header, Client, RequestBuilder, Response, StatusCode};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::sync::broadcast::Receiver;

use super::{MobileBoardBridge, MobileLimits, MobileRoutes, MobileService};
use crate::board::{BoardConfig, BoardService, MemoryBoardStore};
use crate::util::peers::PeerDirectory;
use crate::clipboard::{ClipboardAccess, ClipboardContent};
use crate::constants::mobile::*;
use crate::constants::testing::{EVENT_WAIT, LOOPBACK};
use crate::constants::web::ROUTE_DOWNLOAD;
use crate::error::Result;
use crate::events::{EventBus, NodeEvent};
use crate::lifecycle::Lifecycle;
use crate::net::pinned_client_config;
use crate::server::{HttpServer, RouteProvider};
use crate::util::test_support::shared_test_identity;
use crate::web::WebRoutes;

const PHONE_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Mobile/15E148 Safari/604.1";

#[derive(Default)]
struct FakeClipboard(Mutex<Option<String>>);

impl ClipboardAccess for FakeClipboard {
    fn read_content(&self) -> Result<Option<ClipboardContent>> {
        Ok(self.0.lock().unwrap().clone().map(ClipboardContent::Text))
    }

    fn write_content(&self, content: &ClipboardContent) -> Result<()> {
        if let ClipboardContent::Text(text) = content {
            *self.0.lock().unwrap() = Some(text.clone());
        }
        Ok(())
    }
}

struct NoPeers;

impl PeerDirectory for NoPeers {
    fn peers(&self) -> Vec<crate::device::DeviceInfo> {
        Vec::new()
    }
}

pub(super) struct Fixture {
    pub(super) server: HttpServer,
    pub(super) service: MobileService,
    pub(super) board: Arc<BoardService>,
    _bridge: MobileBoardBridge,
    _board_files: TempDir,
    clipboard: Arc<FakeClipboard>,
    events: Receiver<NodeEvent>,
    pub(super) client: Client,
    downloads: TempDir,
}

impl Fixture {
    pub(super) async fn start() -> Self {
        Self::start_with(MobileLimits::default()).await
    }

    async fn start_with(limits: MobileLimits) -> Self {
        let identity = shared_test_identity();
        let downloads = TempDir::new().unwrap();
        let bus = EventBus::new();
        let events = bus.subscribe();
        let clipboard = Arc::new(FakeClipboard::default());
        let board_files = TempDir::new().unwrap();
        let config = BoardConfig { files_dir: board_files.path().join("cache"), download_dir: downloads.path().to_path_buf(), ..BoardConfig::default() };
        let board = Arc::new(BoardService::new(identity.clone(), Arc::new(MemoryBoardStore::new()), bus.clone(), Arc::new(NoPeers), "Desk".into(), config));
        let service = MobileService::with_limits("Desk".into(), downloads.path().to_path_buf(), bus.clone(), clipboard.clone(), board.clone(), limits);
        let mut bridge = MobileBoardBridge::new(service.clone(), bus);
        bridge.start().await.unwrap();
        let providers: Vec<Arc<dyn RouteProvider>> = vec![Arc::new(MobileRoutes::new(service.clone())), Arc::new(WebRoutes::new())];
        let mut server = HttpServer::new(identity.clone(), providers);
        server.start().await.unwrap();
        let config = pinned_client_config(identity.device_id(), None).unwrap();
        let client = Client::builder().tls_backend_preconfigured(config).redirect(Policy::none()).user_agent(PHONE_AGENT).build().unwrap();
        Self { server, service, board, _bridge: bridge, _board_files: board_files, clipboard, events, client, downloads }
    }

    pub(super) fn url(&self, path: &str) -> String {
        format!("https://{LOOPBACK}:{}{path}", self.server.port().unwrap())
    }

    fn token(&self) -> String {
        self.service.create_invite(vec![format!("https://{LOOPBACK}:1")]).unwrap().token
    }

    async fn redeem(&self, token: &str) -> Response {
        self.client.post(self.url(ROUTE_SESSION)).json(&json!({ "token": token })).send().await.unwrap()
    }

    // Redeems a fresh token and returns the cookie a browser would send back.
    pub(super) async fn login(&self) -> Phone {
        let response = self.redeem(&self.token()).await;
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap().split(';').next().unwrap().to_string();
        let body: Value = response.json().await.unwrap();
        Phone { cookie, session_id: body["sessionId"].as_str().unwrap().to_string() }
    }

    pub(super) fn get(&self, phone: &Phone, path: &str) -> RequestBuilder {
        self.client.get(self.url(path)).header(header::COOKIE, &phone.cookie)
    }

    pub(super) fn post(&self, phone: &Phone, path: &str) -> RequestBuilder {
        self.client.post(self.url(path)).header(header::COOKIE, &phone.cookie)
    }

    async fn next_event(&mut self, wanted: impl Fn(&NodeEvent) -> bool) -> NodeEvent {
        tokio::time::timeout(EVENT_WAIT, async {
            loop {
                let event = self.events.recv().await.unwrap();
                if wanted(&event) {
                    return event;
                }
            }
        })
        .await
        .expect("expected event did not arrive")
    }
}

pub(super) struct Phone {
    pub(super) cookie: String,
    pub(super) session_id: String,
}

#[tokio::test]
async fn redeeming_a_token_sets_a_hardened_cookie_and_lists_the_session() {
    let mut fx = Fixture::start().await;
    let response = fx.redeem(&fx.token()).await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap().to_string();
    for part in ["muniko_session=", "HttpOnly", "Secure", "SameSite=Strict", "Path=/"] {
        assert!(cookie.contains(part), "{cookie}");
    }
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["deviceName"], "Desk");
    let sessions = fx.service.sessions();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, body["sessionId"].as_str().unwrap());
    assert_eq!(sessions[0].label, "iPhone - Safari");
    fx.next_event(|e| matches!(e, NodeEvent::PhoneConnected { .. })).await;
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn token_is_single_use() {
    let mut fx = Fixture::start().await;
    let token = fx.token();
    assert_eq!(fx.redeem(&token).await.status(), StatusCode::OK);
    assert_eq!(fx.redeem(&token).await.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(fx.redeem("not-a-token").await.status(), StatusCode::UNAUTHORIZED);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn expired_token_is_rejected() {
    let mut fx = Fixture::start_with(MobileLimits { token_ttl_secs: 0, ..MobileLimits::default() }).await;
    assert_eq!(fx.redeem(&fx.token()).await.status(), StatusCode::UNAUTHORIZED);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn redemption_is_rate_limited_per_ip() {
    let mut fx = Fixture::start_with(MobileLimits { max_failed_redeems: 3, ..MobileLimits::default() }).await;
    for _ in 0..3 {
        assert_eq!(fx.redeem("bad").await.status(), StatusCode::UNAUTHORIZED);
    }
    assert_eq!(fx.redeem(&fx.token()).await.status(), StatusCode::TOO_MANY_REQUESTS);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn every_api_endpoint_needs_a_session() {
    let mut fx = Fixture::start().await;
    let client = &fx.client;
    let requests = [
        client.get(fx.url(ROUTE_SESSION)),
        client.delete(fx.url(ROUTE_SESSION)),
        client.post(fx.url(ROUTE_UPLOAD)).header(FILE_NAME_HEADER, "a.txt").body("x"),
        client.get(fx.url(ROUTE_INBOX)),
        client.get(fx.url("/api/m/inbox/abc")),
        client.post(fx.url(ROUTE_TEXT)).body(r#"{"text":"x"}"#),
        client.get(fx.url(ROUTE_TEXT)),
        client.get(fx.url(ROUTE_EVENTS)),
        client.get(fx.url(ROUTE_EVENTS)).header(header::COOKIE, "muniko_session=forged"),
    ];
    for request in requests {
        assert_eq!(request.send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    }
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn session_check_and_logout() {
    let mut fx = Fixture::start().await;
    let phone = fx.login().await;
    assert_eq!(fx.get(&phone, ROUTE_SESSION).send().await.unwrap().status(), StatusCode::OK);
    let response = fx.client.delete(fx.url(ROUTE_SESSION)).header(header::COOKIE, &phone.cookie).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(fx.get(&phone, ROUTE_SESSION).send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    fx.next_event(|e| matches!(e, NodeEvent::PhoneDisconnected { .. })).await;
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn upload_lands_in_downloads_with_a_safe_unique_name() {
    let mut fx = Fixture::start().await;
    let phone = fx.login().await;
    let send = |name: &'static str| fx.post(&phone, ROUTE_UPLOAD).header(FILE_NAME_HEADER, name).body("hello phone").send();
    let first = send("..%2F..%2Fphoto%20one.txt").await.unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);
    assert_eq!(first.json::<Value>().await.unwrap()["name"], "photo one.txt");
    let second: Value = send("photo%20one.txt").await.unwrap().json().await.unwrap();
    assert_eq!(second["name"], "photo one (1).txt");
    assert_eq!(std::fs::read_to_string(fx.downloads.path().join("photo one.txt")).unwrap(), "hello phone");
    let expected = format!("{PHONE_ID_PREFIX}{}", phone.session_id);
    let event = fx.next_event(|e| matches!(e, NodeEvent::FileReceived { .. })).await;
    assert!(matches!(event, NodeEvent::FileReceived { from_device_id, .. } if from_device_id == expected));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn upload_rejects_bad_names_empty_and_oversized_bodies() {
    let mut fx = Fixture::start_with(MobileLimits { max_upload_bytes: 4, ..MobileLimits::default() }).await;
    let phone = fx.login().await;
    assert_eq!(fx.post(&phone, ROUTE_UPLOAD).body("x").send().await.unwrap().status(), StatusCode::BAD_REQUEST);
    assert_eq!(fx.post(&phone, ROUTE_UPLOAD).header(FILE_NAME_HEADER, "..").body("x").send().await.unwrap().status(), StatusCode::BAD_REQUEST);
    assert_eq!(fx.post(&phone, ROUTE_UPLOAD).header(FILE_NAME_HEADER, "a.txt").send().await.unwrap().status(), StatusCode::BAD_REQUEST);
    let big = fx.post(&phone, ROUTE_UPLOAD).header(FILE_NAME_HEADER, "a.txt").body("too big").send().await.unwrap();
    assert_eq!(big.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(std::fs::read_dir(fx.downloads.path()).unwrap().count(), 0);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn inbox_lists_streams_and_supports_range() {
    let mut fx = Fixture::start().await;
    let phone = fx.login().await;
    let source = TempDir::new().unwrap();
    let file = source.path().join("notes.txt");
    std::fs::write(&file, "0123456789").unwrap();
    let added = fx.service.send_files(&phone.session_id, vec![file, source.path().to_path_buf()]).await.unwrap();
    assert_eq!(added, 1);

    let list: Value = fx.get(&phone, ROUTE_INBOX).send().await.unwrap().json().await.unwrap();
    assert_eq!(list[0]["name"], "notes.txt");
    assert_eq!(list[0]["size"], 10);
    let id = list[0]["id"].as_str().unwrap();

    let whole = fx.get(&phone, &format!("{ROUTE_INBOX}/{id}")).send().await.unwrap();
    assert_eq!(whole.status(), StatusCode::OK);
    assert!(whole.headers()[header::CONTENT_DISPOSITION].to_str().unwrap().starts_with("attachment"));
    assert_eq!(whole.text().await.unwrap(), "0123456789");

    let part = fx.get(&phone, &format!("{ROUTE_INBOX}/{id}")).header(header::RANGE, "bytes=3-5").send().await.unwrap();
    assert_eq!(part.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(part.headers()[header::CONTENT_RANGE], "bytes 3-5/10");
    assert_eq!(part.text().await.unwrap(), "345");

    let bad = fx.get(&phone, &format!("{ROUTE_INBOX}/{id}")).header(header::RANGE, "bytes=50-").send().await.unwrap();
    assert_eq!(bad.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(fx.get(&phone, "/api/m/inbox/missing").send().await.unwrap().status(), StatusCode::NOT_FOUND);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn inbox_items_expire_and_unknown_sessions_are_refused() {
    let mut fx = Fixture::start_with(MobileLimits { inbox_ttl_secs: 0, ..MobileLimits::default() }).await;
    let phone = fx.login().await;
    let source = TempDir::new().unwrap();
    let file = source.path().join("a.txt");
    std::fs::write(&file, "a").unwrap();
    fx.service.send_files(&phone.session_id, vec![file]).await.unwrap();
    let list: Value = fx.get(&phone, ROUTE_INBOX).send().await.unwrap().json().await.unwrap();
    assert_eq!(list.as_array().unwrap().len(), 0);
    assert!(fx.service.send_files("unknown", vec![]).await.is_err());
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn text_goes_up_to_the_clipboard_and_down_to_the_phone() {
    let mut fx = Fixture::start().await;
    let phone = fx.login().await;
    let up = fx.post(&phone, ROUTE_TEXT).body(r#"{"text":"from phone"}"#).send().await.unwrap();
    assert_eq!(up.status(), StatusCode::NO_CONTENT);
    assert_eq!(fx.clipboard.0.lock().unwrap().as_deref(), Some("from phone"));
    let event = fx.next_event(|e| matches!(e, NodeEvent::ClipboardReceived { .. })).await;
    assert!(matches!(event, NodeEvent::ClipboardReceived { text, from_device_id } if text == "from phone" && from_device_id.starts_with(PHONE_ID_PREFIX)));
    assert_eq!(fx.post(&phone, ROUTE_TEXT).body(r#"{"text":""}"#).send().await.unwrap().status(), StatusCode::BAD_REQUEST);
    let huge = json!({ "text": "a".repeat(MAX_TEXT_BYTES + 1) }).to_string();
    assert_eq!(fx.post(&phone, ROUTE_TEXT).body(huge).send().await.unwrap().status(), StatusCode::PAYLOAD_TOO_LARGE);

    assert_eq!(fx.get(&phone, ROUTE_TEXT).send().await.unwrap().status(), StatusCode::NO_CONTENT);
    fx.service.share_text(&phone.session_id, "from desktop".into()).unwrap();
    let down: Value = fx.get(&phone, ROUTE_TEXT).send().await.unwrap().json().await.unwrap();
    assert_eq!(down["text"], "from desktop");
    assert!(down["sentAt"].as_u64().unwrap() > 0);
    fx.server.stop().await.unwrap();
}

pub(super) async fn read_until(response: &mut Response, needle: &str) -> String {
    let mut seen = String::new();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !seen.contains(needle) {
            match response.chunk().await.unwrap() {
                Some(chunk) => seen.push_str(&String::from_utf8_lossy(&chunk)),
                None => break,
            }
        }
    })
    .await
    .expect("stream stalled");
    seen
}

#[tokio::test]
async fn events_stream_reports_updates_and_revocation() {
    let mut fx = Fixture::start().await;
    let phone = fx.login().await;
    let mut stream = fx.get(&phone, ROUTE_EVENTS).send().await.unwrap();
    assert_eq!(stream.status(), StatusCode::OK);
    assert!(stream.headers()[header::CONTENT_TYPE].to_str().unwrap().starts_with("text/event-stream"));
    fx.service.share_text(&phone.session_id, "hi".into()).unwrap();
    assert!(read_until(&mut stream, "event: text").await.contains("event: text"));

    assert!(fx.service.revoke(&phone.session_id));
    assert!(read_until(&mut stream, "event: revoked").await.contains("event: revoked"));
    assert_eq!(fx.get(&phone, ROUTE_INBOX).send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    fx.next_event(|e| matches!(e, NodeEvent::PhoneDisconnected { .. })).await;
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn disabling_ends_sessions_and_hides_the_page() {
    let mut fx = Fixture::start().await;
    let phone = fx.login().await;
    let page = fx.client.get(fx.url(ROUTE_MOBILE_DIR)).send().await.unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    assert!(page.headers()[header::CONTENT_TYPE].to_str().unwrap().starts_with("text/html"));

    fx.service.set_enabled(false);
    assert!(!fx.service.is_enabled());
    assert!(fx.service.sessions().is_empty());
    assert!(fx.service.create_invite(vec![]).is_err());
    assert_eq!(fx.get(&phone, ROUTE_INBOX).send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    for path in [ROUTE_MOBILE_DIR, "/m/anything"] {
        let response = fx.client.get(fx.url(path)).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], ROUTE_DOWNLOAD);
    }
    fx.service.set_enabled(true);
    assert_eq!(fx.client.get(fx.url(ROUTE_MOBILE_DIR)).send().await.unwrap().status(), StatusCode::OK);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn spa_falls_back_to_index_for_routes_but_not_for_missing_files() {
    let mut fx = Fixture::start().await;
    let route = fx.client.get(fx.url("/m/settings")).send().await.unwrap();
    assert_eq!(route.status(), StatusCode::OK);
    assert_eq!(route.headers()[header::CACHE_CONTROL], CACHE_NO_CACHE);
    assert_eq!(fx.client.get(fx.url("/m/missing.js")).send().await.unwrap().status(), StatusCode::NOT_FOUND);
    assert_eq!(fx.client.get(fx.url(ROUTE_MOBILE)).send().await.unwrap().status(), StatusCode::PERMANENT_REDIRECT);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn browser_root_still_redirects_to_download() {
    let mut fx = Fixture::start().await;
    let response = fx.client.get(fx.url("/")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], ROUTE_DOWNLOAD);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn sweeper_expires_sessions() {
    let mut fx = Fixture::start_with(MobileLimits { session_ttl_secs: 0, ..MobileLimits::default() }).await;
    let phone = fx.login().await;
    let mut sweeper = super::MobileSweeper::new(fx.service.clone());
    sweeper.start().await.unwrap();
    fx.next_event(|e| matches!(e, NodeEvent::PhoneDisconnected { .. })).await;
    sweeper.stop().await.unwrap();
    assert!(fx.service.sessions().is_empty());
    assert_eq!(fx.get(&phone, ROUTE_INBOX).send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    fx.server.stop().await.unwrap();
}
