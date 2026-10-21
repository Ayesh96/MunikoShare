//! Fakes and fixtures shared by the clipboard tests; never touches the real clipboard.

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::content::ClipboardContent;
use super::{png_codec, ClipboardAccess, ClipboardRoutes, ClipboardShared, SensitivityProbe};
use crate::constants::testing::LOOPBACK;
use crate::device::DeviceInfo;
use crate::error::Result;
use crate::events::EventBus;
use crate::identity::Identity;
use crate::server::{HttpServer, RouteProvider};
use crate::lifecycle::Lifecycle;
use crate::trust::{MemoryTrustStore, TrustStore, TrustedDevice};
use crate::util::peers::PeerDirectory;
use crate::util::test_support::shared_test_identity;

#[derive(Default)]
pub struct FakeClipboard {
    content: Mutex<Option<ClipboardContent>>,
    writes: AtomicUsize,
}

impl FakeClipboard {
    pub fn with(content: ClipboardContent) -> Arc<Self> {
        Arc::new(Self { content: Mutex::new(Some(content)), writes: AtomicUsize::new(0) })
    }

    pub fn empty() -> Arc<Self> {
        Arc::default()
    }

    pub fn current(&self) -> Option<ClipboardContent> {
        self.content.lock().unwrap().clone()
    }

    pub fn text(&self) -> Option<String> {
        match self.current() {
            Some(ClipboardContent::Text(text)) => Some(text),
            _ => None,
        }
    }

    /// Simulates the user copying something locally (not counted as a write by a peer).
    pub fn user_copy(&self, content: ClipboardContent) {
        *self.content.lock().unwrap() = Some(content);
    }

    pub fn clear(&self) {
        *self.content.lock().unwrap() = None;
    }

    pub fn writes(&self) -> usize {
        self.writes.load(Ordering::SeqCst)
    }
}

impl ClipboardAccess for FakeClipboard {
    fn read_content(&self) -> Result<Option<ClipboardContent>> {
        Ok(self.current())
    }

    fn write_content(&self, content: &ClipboardContent) -> Result<()> {
        *self.content.lock().unwrap() = Some(content.clone());
        self.writes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeProbe(pub AtomicBool);

impl SensitivityProbe for FakeProbe {
    fn is_sensitive(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Default)]
pub struct FixedPeers(pub Mutex<Vec<DeviceInfo>>);

impl PeerDirectory for FixedPeers {
    fn peers(&self) -> Vec<DeviceInfo> {
        self.0.lock().unwrap().clone()
    }
}

pub fn test_image(width: u32, height: u32) -> ClipboardContent {
    let pixels: Vec<u8> = (0..width * height * 4).map(|i| (i % 251) as u8).collect();
    ClipboardContent::Image { width, height, png: png_codec::encode(width, height, &pixels).unwrap() }
}

/// A device with a loopback server running the clipboard routes.
pub struct TestNode {
    pub identity: Arc<Identity>,
    pub clipboard: Arc<FakeClipboard>,
    pub trust: Arc<MemoryTrustStore>,
    pub events: EventBus,
    pub shared: ClipboardShared,
    pub server: HttpServer,
}

impl TestNode {
    pub async fn start(clipboard: Arc<FakeClipboard>) -> Self {
        let identity = shared_test_identity();
        let trust = Arc::new(MemoryTrustStore::new());
        let events = EventBus::new();
        let shared = ClipboardShared::new(clipboard.clone());
        let routes = ClipboardRoutes::with_shared(clipboard.clone(), trust.clone(), events.clone(), shared.clone());
        let providers: Vec<Arc<dyn RouteProvider>> = vec![Arc::new(routes)];
        let mut server = HttpServer::new(identity.clone(), providers);
        server.start().await.unwrap();
        Self { identity, clipboard, trust, events, shared, server }
    }

    pub fn trust_device(&self, other: &TestNode, sync: bool) {
        let id = other.identity.device_id().to_string();
        self.trust.add(TrustedDevice { id: id.clone(), name: "peer".into(), os: "test".into(), paired_at: 0 }).unwrap();
        self.trust.set_clipboard_sync(&id, sync).unwrap();
    }

    pub fn info(&self) -> DeviceInfo {
        DeviceInfo {
            id: self.identity.device_id().to_string(),
            name: "peer".into(),
            os: "test".into(),
            addresses: vec![LOOPBACK.parse::<IpAddr>().unwrap()],
            port: self.server.port().unwrap(),
            protocol_version: 1,
        }
    }
}
