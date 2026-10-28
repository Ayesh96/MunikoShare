//! MunikoNode owns every service and drives their lifecycle in order.

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::broadcast;

use crate::board::{BoardConfig, BoardRuntime, BoardService, SqliteBoardStore};
use crate::clipboard::{
    ClipboardAccess, ClipboardRoutes, ClipboardService, ClipboardShared, ClipboardWatcher, HistorySummary, SystemClipboard,
};
use crate::constants::app::{DATABASE_FILE_NAME, PROTOCOL_VERSION};
use crate::device::DeviceInfo;
use crate::discovery::DiscoveryService;
use crate::error::{MunikoError, Result};
use crate::events::{EventBus, NodeEvent};
use crate::history::{HistoryEntry, HistoryRecorder, HistoryStore, SqliteHistoryStore};
use crate::identity::{Identity, IdentityStore};
use crate::info::{InfoRoutes, LocalDevice};
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::logs;
use crate::mobile::{Invite, MobileBoardBridge, MobileRoutes, MobileService, MobileSweeper, PhoneSession};
use crate::net::probe;
use crate::pairing::{PairingProfile, PairingService};
use crate::server::{HttpServer, RouteProvider};
use crate::transfer::{OfferResponder, TransferClient, TransferRoutes};
use crate::trust::{SqliteTrustStore, TrustStore, TrustedDevice};
use crate::util::data_url::png_data_url;
use crate::util::fs::ensure_dir;
use crate::util::peers::{LivePeers, PeerDirectory};
use crate::web::WebRoutes;

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub device_name: String,
    pub data_dir: PathBuf,
    pub download_dir: PathBuf,
}

pub struct MunikoNode {
    config: NodeConfig,
    identity: Arc<Identity>,
    events: EventBus,
    server: HttpServer,
    client: TransferClient,
    offers: OfferResponder,
    pairing: PairingService,
    trust: Arc<dyn TrustStore>,
    clipboard: ClipboardService,
    clipboard_shared: ClipboardShared,
    watcher: ClipboardWatcher,
    live_peers: Arc<LivePeers>,
    board: Arc<BoardService>,
    board_runtime: BoardRuntime,
    mobile: MobileService,
    mobile_sweeper: MobileSweeper,
    mobile_board_bridge: MobileBoardBridge,
    history: Arc<dyn HistoryStore>,
    recorder: HistoryRecorder,
    local: LocalDevice,
    discovery: Option<DiscoveryService>,
    state: LifecycleState,
    #[cfg(test)]
    fail_start: bool,
}

impl MunikoNode {
    pub async fn new(config: NodeConfig) -> Result<Self> {
        ensure_dir(&config.data_dir).await?;
        ensure_dir(&config.download_dir).await?;
        let identity = Arc::new(IdentityStore::new(config.data_dir.clone()).load_or_create().await?);
        let events = EventBus::new();
        let database = config.data_dir.join(DATABASE_FILE_NAME);
        let trust: Arc<dyn TrustStore> = Arc::new(SqliteTrustStore::open(&database)?);
        let history: Arc<dyn HistoryStore> = Arc::new(SqliteHistoryStore::open(&database)?);
        let recorder = HistoryRecorder::new(history.clone(), events.clone());
        let clipboard_access: Arc<dyn ClipboardAccess> = Arc::new(SystemClipboard::new());
        let profile = PairingProfile { name: config.device_name.clone(), os: std::env::consts::OS.to_string() };

        let transfers = TransferRoutes::new(config.download_dir.clone(), events.clone(), trust.clone());
        let offers = transfers.responder();
        let local = LocalDevice::new(DeviceInfo {
            id: identity.device_id().to_string(),
            name: profile.name.clone(),
            os: profile.os.clone(),
            addresses: Vec::new(),
            port: 0,
            protocol_version: PROTOCOL_VERSION,
        });
        let pairing = PairingService::new(identity.clone(), trust.clone(), events.clone(), profile);
        let clipboard_shared = ClipboardShared::new(clipboard_access.clone());
        let live_peers = Arc::new(LivePeers::new());
        let board = Arc::new(BoardService::new(
            identity.clone(),
            Arc::new(SqliteBoardStore::open(&database)?),
            events.clone(),
            live_peers.clone(),
            config.device_name.clone(),
            BoardConfig::in_dirs(&config.data_dir, config.download_dir.clone()),
        ));
        let board_runtime = board.runtime();
        let mobile = MobileService::new(
            config.device_name.clone(),
            config.download_dir.clone(),
            events.clone(),
            clipboard_access.clone(),
            board.clone(),
        );
        let mobile_board_bridge = MobileBoardBridge::new(mobile.clone(), events.clone());
        let mobile_sweeper = MobileSweeper::new(mobile.clone());
        let watcher = ClipboardWatcher::new(
            identity.clone(),
            clipboard_access.clone(),
            trust.clone(),
            live_peers.clone(),
            events.clone(),
            clipboard_shared.clone(),
        );
        let providers: Vec<Arc<dyn RouteProvider>> = vec![
            Arc::new(transfers),
            Arc::new(pairing.routes()),
            Arc::new(ClipboardRoutes::with_shared(
                clipboard_access.clone(),
                trust.clone(),
                events.clone(),
                clipboard_shared.clone(),
            )),
            Arc::new(InfoRoutes::new(local.clone())),
            Arc::new(board.routes()),
            Arc::new(MobileRoutes::new(mobile.clone())),
            Arc::new(WebRoutes::new()),
        ];
        let server = HttpServer::new(identity.clone(), providers);
        let client = TransferClient::new(identity.clone(), events.clone());
        let clipboard = ClipboardService::new(identity.clone(), clipboard_access);
        Ok(Self {
            config,
            identity,
            events,
            server,
            client,
            offers,
            pairing,
            trust,
            clipboard,
            clipboard_shared,
            watcher,
            live_peers,
            board,
            board_runtime,
            mobile,
            mobile_sweeper,
            mobile_board_bridge,
            history,
            recorder,
            local,
            discovery: None,
            state: LifecycleState::Created,
            #[cfg(test)]
            fail_start: false,
        })
    }

    /// This device as other peers see it (addresses are filled in by discovery).
    pub fn local_device(&self) -> DeviceInfo {
        DeviceInfo {
            id: self.identity.device_id().to_string(),
            name: self.config.device_name.clone(),
            os: std::env::consts::OS.to_string(),
            addresses: Vec::new(),
            port: self.server.port().unwrap_or_default(),
            protocol_version: PROTOCOL_VERSION,
        }
    }

    pub fn peers(&self) -> Vec<DeviceInfo> {
        self.live_peers.peers()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NodeEvent> {
        self.events.subscribe()
    }

    /// Offers files to a peer; the transfer client publishes progress and outcome events.
    pub async fn send_files(&self, peer_id: &str, paths: &[PathBuf]) -> Result<()> {
        let peer = self.find_peer(peer_id)?;
        self.client.send_files(&peer, paths).await
    }

    pub fn respond_to_offer(&self, offer_id: &str, accept: bool) -> Result<()> {
        self.offers.respond_to_offer(offer_id, accept)
    }

    /// Asks a peer to show a pairing code; the user then types it here via confirm_pairing.
    pub async fn request_pairing(&self, peer_id: &str) -> Result<()> {
        self.pairing.request(&self.find_peer(peer_id)?).await
    }

    pub async fn confirm_pairing(&self, peer_id: &str, code: &str) -> Result<()> {
        self.pairing.confirm(&self.find_peer(peer_id)?, code).await
    }

    pub fn unpair(&self, device_id: &str) -> Result<()> {
        self.pairing.unpair(device_id)
    }

    pub fn trusted_devices(&self) -> Vec<TrustedDevice> {
        self.pairing.trusted()
    }

    /// Sends this device's current clipboard text to a peer.
    pub async fn push_clipboard(&self, peer_id: &str) -> Result<()> {
        self.clipboard.push_current(&self.find_peer(peer_id)?).await?;
        self.events.publish(NodeEvent::ClipboardSent { to_device_id: peer_id.to_string() });
        Ok(())
    }

    pub async fn push_text(&self, peer_id: &str, text: &str) -> Result<()> {
        self.clipboard.push_text(&self.find_peer(peer_id)?, text).await?;
        self.events.publish(NodeEvent::ClipboardSent { to_device_id: peer_id.to_string() });
        Ok(())
    }

    pub fn set_auto_accept(&self, device_id: &str, enabled: bool) -> Result<()> {
        self.trust.set_auto_accept(device_id, enabled)
    }

    pub fn auto_accept(&self, device_id: &str) -> bool {
        self.trust.auto_accept(device_id)
    }

    /// Newest first.
    pub fn history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        self.history.recent(limit)
    }

    pub fn clear_history(&self) -> Result<()> {
        self.history.clear()
    }

    /// Adds a device that discovery cannot see, after verifying it is a Muniko device.
    pub async fn add_peer_by_address(&self, ip: IpAddr, port: u16) -> Result<DeviceInfo> {
        let discovery = self.discovery.as_ref().ok_or_else(|| MunikoError::InvalidState(format!("{:?}", self.state)))?;
        let device = probe(&self.identity, ip, port).await?;
        discovery.add_manual_peer(device.clone());
        Ok(device)
    }

    pub fn set_clipboard_sync(&self, device_id: &str, enabled: bool) -> Result<()> {
        self.trust.set_clipboard_sync(device_id, enabled)
    }

    pub fn clipboard_sync(&self, device_id: &str) -> bool {
        self.trust.clipboard_sync(device_id)
    }

    /// Recent clipboard items, newest first (previews only).
    pub fn clipboard_history(&self) -> Vec<HistorySummary> {
        self.clipboard_shared.history.list()
    }

    pub async fn copy_clipboard_item(&self, id: &str) -> Result<()> {
        self.clipboard_shared.history.copy_back(id).await
    }

    pub fn clipboard_image(&self, id: &str) -> Option<String> {
        self.clipboard_shared.history.image_data_url(id)
    }

    pub fn pin_clipboard_item(&self, id: &str, pinned: bool) -> bool {
        self.clipboard_shared.history.set_pinned(id, pinned)
    }

    pub fn clear_clipboard_history(&self) {
        self.clipboard_shared.history.clear()
    }

    /// Puts a board image on the system clipboard, fetching it from a peer first if needed.
    pub async fn copy_board_image(&self, item_id: &str) -> Result<()> {
        self.clipboard.write_png(self.board.read_file(item_id).await?).await
    }

    /// A board image as a data URL for previews, fetched from a peer first if needed.
    pub async fn board_image(&self, item_id: &str) -> Result<String> {
        Ok(png_data_url(&self.board.read_file(item_id).await?))
    }

    /// The Network Board facade; the desktop app calls it directly.
    pub fn board(&self) -> Arc<BoardService> {
        self.board.clone()
    }

    /// One-time QR link per LAN address; the desktop supplies its addresses.
    pub fn create_phone_invite(&self, addresses: &[IpAddr]) -> Result<Invite> {
        let port = self.server.port().ok_or_else(|| MunikoError::InvalidState(format!("{:?}", self.state)))?;
        let base_urls = addresses.iter().filter(|ip| !ip.is_loopback()).map(|ip| https_base_url(*ip, port)).collect();
        self.mobile.create_invite(base_urls)
    }

    pub fn phone_sessions(&self) -> Vec<PhoneSession> {
        self.mobile.sessions()
    }

    pub fn revoke_phone(&self, session_id: &str) -> bool {
        self.mobile.revoke(session_id)
    }

    pub async fn send_files_to_phone(&self, session_id: &str, paths: Vec<PathBuf>) -> Result<usize> {
        self.mobile.send_files(session_id, paths).await
    }

    pub fn share_text_to_phone(&self, session_id: &str, text: String) -> Result<()> {
        self.mobile.share_text(session_id, text)
    }

    pub fn set_phone_access(&self, enabled: bool) {
        self.mobile.set_enabled(enabled)
    }

    pub fn phone_access(&self) -> bool {
        self.mobile.is_enabled()
    }

    fn find_peer(&self, peer_id: &str) -> Result<DeviceInfo> {
        self.peers()
            .into_iter()
            .find(|peer| peer.id == peer_id)
            .ok_or_else(|| MunikoError::PeerNotFound(peer_id.to_string()))
    }
}

#[async_trait::async_trait]
impl Lifecycle for MunikoNode {
    async fn start(&mut self) -> Result<()> {
        if self.state == LifecycleState::Running {
            return Err(MunikoError::InvalidState(format!("{:?}", self.state)));
        }
        if let Err(error) = self.start_services().await {
            // Release whatever already started so no port or task leaks.
            if let Err(cleanup) = self.stop_services().await {
                tracing::warn!(%cleanup, "{}", logs::NODE_CLEANUP_FAILED);
            }
            self.state = LifecycleState::Stopped;
            return Err(error);
        }
        self.state = LifecycleState::Running;
        tracing::info!("{}", logs::NODE_STARTED);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let stopped = self.stop_services().await;
        self.state = LifecycleState::Stopped;
        tracing::info!("{}", logs::NODE_STOPPED);
        stopped
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

impl MunikoNode {
    async fn start_services(&mut self) -> Result<()> {
        // Recorder first so it sees every event; server before discovery so the real port is announced.
        self.recorder.start().await?;
        self.server.start().await?;
        self.mobile_sweeper.start().await?;
        self.mobile_board_bridge.start().await?;
        self.local.set_port(self.server.port().unwrap_or_default());
        let mut discovery = DiscoveryService::new(self.local_device(), self.events.clone());
        discovery.start().await?;
        self.live_peers.attach(discovery.directory());
        self.discovery = Some(discovery);
        self.watcher.start().await?;
        self.board_runtime.start().await?;
        #[cfg(test)]
        if self.fail_start {
            return Err(MunikoError::InvalidState(logs::NODE_INJECTED_FAILURE.to_string()));
        }
        Ok(())
    }

    /// Reverse of start (stop announcing before closing the server); every service is stopped even if one fails.
    async fn stop_services(&mut self) -> Result<()> {
        let mut first = None;
        keep_first(&mut first, self.board_runtime.stop().await);
        keep_first(&mut first, self.watcher.stop().await);
        self.live_peers.detach();
        if let Some(mut discovery) = self.discovery.take() {
            keep_first(&mut first, discovery.stop().await);
        }
        keep_first(&mut first, self.mobile_board_bridge.stop().await);
        keep_first(&mut first, self.mobile_sweeper.stop().await);
        keep_first(&mut first, self.server.stop().await);
        keep_first(&mut first, self.recorder.stop().await);
        first.map_or(Ok(()), Err)
    }
}

fn keep_first(first: &mut Option<MunikoError>, result: Result<()>) {
    if let Err(error) = result {
        first.get_or_insert(error);
    }
}

fn https_base_url(ip: IpAddr, port: u16) -> String {
    match ip {
        IpAddr::V4(v4) => format!("https://{v4}:{port}"),
        IpAddr::V6(v6) => format!("https://[{v6}]:{port}"),
    }
}

#[cfg(test)]
#[path = "node_tests.rs"]
mod tests;
