use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::bootstrap::BoardBootstrap;
use super::client::BoardClient;
use super::config::BoardConfig;
use super::context::BoardContext;
use super::files::{ensure_cached, export_to_downloads, publish_if_changed, reconcile, Availability, BoardFiles, FileCache, FileLease, FileWorker};
use super::membership::Membership;
use super::model::{Board, BoardItem, BoardMode, ItemKind};
use super::observed::{ObservedBoard, ObservedBoards};
use super::outbox::Outbox;
use super::replica::Replica;
use super::routes::BoardRoutes;
use super::runtime::BoardRuntime;
use super::signing::ItemSigner;
use super::store::BoardStore;
use super::syncer::BoardSyncer;
use super::limits::valid_file_name;
use super::view::{build_views, view_of, BoardItemView, ViewKind};
use crate::constants::board::MAX_TEXT_BYTES;
use crate::board::BoardError;
use crate::error::{MunikoError, Result};
use crate::events::EventBus;
use crate::identity::Identity;
use crate::messages::board as msg;
use crate::util::peers::PeerDirectory;
use crate::util::time::unix_now;

/// A cached board file that stays on disk while this value (or its lease) is alive.
pub struct OpenedFile {
    pub path: PathBuf,
    pub view: BoardItemView,
    pub lease: FileLease,
}

/// Facade of the Network Board used by the node and the UI.
pub struct BoardService {
    ctx: Arc<BoardContext>,
}

impl BoardService {
    /// Needs this device's identity (signs items), the store (may share muniko.db), the event bus,
    /// the peer directory and the local device name shown to other members.
    pub fn new(
        identity: Arc<Identity>,
        store: Arc<dyn BoardStore>,
        events: EventBus,
        peers: Arc<dyn PeerDirectory>,
        device_name: String,
        config: BoardConfig,
    ) -> Self {
        let ctx = BoardContext {
            signer: ItemSigner::new(identity.clone()),
            replica: Arc::new(Replica::new(store, config.retention)),
            client: BoardClient::new(identity),
            events,
            peers,
            outbox: Arc::new(Outbox::default()),
            observed: ObservedBoards::default(),
            files: BoardFiles::new(Arc::new(FileCache::new(config.files_dir.clone()))),
            device_name,
            config,
        };
        Self { ctx: Arc::new(ctx) }
    }

    /// Route provider to register on the shared HTTPS server.
    pub fn routes(&self) -> BoardRoutes {
        BoardRoutes::new(self.ctx.replica.clone(), self.ctx.events.clone(), self.ctx.outbox.clone(), self.ctx.files.cache.clone())
    }

    /// Background syncer and bootstrap; start after the server and discovery, stop before them.
    pub fn runtime(&self) -> BoardRuntime {
        BoardRuntime::new(BoardSyncer::new(self.ctx.clone()), FileWorker::new(self.ctx.clone()), BoardBootstrap::new(self.ctx.clone()))
    }

    pub fn board(&self) -> Result<Option<Board>> {
        Ok(self.ctx.replica.record()?.map(|record| record.board))
    }

    /// Visible items, newest first, with author names resolved.
    pub fn items(&self) -> Result<Vec<BoardItemView>> {
        let replica = &self.ctx.replica;
        let items = replica.store().all()?;
        Ok(build_views(items, &replica.members()?, &replica.tombstone_authors()?, self.ctx.local_id(), &Availability::of(&self.ctx)))
    }

    pub fn post_text(&self, text: &str) -> Result<BoardItemView> {
        if text.trim().is_empty() {
            return Err(MunikoError::from(BoardError::TextEmpty));
        }
        if text.len() > MAX_TEXT_BYTES {
            return Err(MunikoError::from(BoardError::TextTooLarge));
        }
        self.post(ItemKind::Text { text: text.to_string() })
    }

    /// Copies the file into this device's cache (so it can serve it) and posts its name, size and SHA-256.
    pub async fn post_file(&self, path: &Path) -> Result<BoardItemView> {
        let name = path.file_name().and_then(|n| n.to_str()).filter(|n| valid_file_name(n)).ok_or_else(|| MunikoError::from(BoardError::InvalidFileName))?;
        if !tokio::fs::metadata(path).await.map_err(|e| MunikoError::from(BoardError::storage(msg::READ_FILE_FAILED, e)))?.is_file() {
            return Err(MunikoError::from(BoardError::NotAFile));
        }
        // The guard keeps cache cleanup off the file until the item is stored.
        let (ingested, _guard) = self.ctx.files.cache.ingest(path).await?;
        self.post(ItemKind::File { name: name.to_string(), size: ingested.size, sha256: ingested.sha256 })
    }

    /// Posts in-memory content (such as a pasted image) as a file called `name`.
    pub async fn post_bytes(&self, name: &str, bytes: &[u8]) -> Result<BoardItemView> {
        if !valid_file_name(name) {
            return Err(MunikoError::from(BoardError::InvalidFileName));
        }
        let (ingested, _guard) = self.ctx.files.cache.ingest_bytes(bytes).await?;
        self.post(ItemKind::File { name: name.to_string(), size: ingested.size, sha256: ingested.sha256 })
    }

    /// Fetches the file into the cache if needed (from any online holder) and exports it to the download folder.
    pub async fn download_file(&self, item_id: &str) -> Result<PathBuf> {
        let exported = export_to_downloads(&self.ctx, item_id).await?;
        reconcile(&self.ctx)?;
        Ok(exported)
    }

    /// The file's bytes, fetched into the cache first if needed.
    pub async fn read_file(&self, item_id: &str) -> Result<Vec<u8>> {
        let opened = self.open_file(item_id).await?;
        tokio::fs::read(&opened.path).await.map_err(|e| MunikoError::from(BoardError::storage(msg::READ_FILE_FAILED, e)))
    }

    /// Fetches the file into the cache if needed (from any online holder) and returns it pinned until the lease drops.
    pub async fn open_file(&self, item_id: &str) -> Result<OpenedFile> {
        let view = self.items()?.into_iter().find(|view| view.id == item_id).ok_or_else(|| MunikoError::from(BoardError::ItemNotFound))?;
        let ViewKind::File { size, sha256, .. } = &view.kind else { return Err(MunikoError::from(BoardError::NotAFileItem)) };
        let holders = self.ctx.files.holders.of(sha256, &self.ctx.peers.peers());
        let _guard = ensure_cached(&self.ctx, sha256, *size, &holders).await?;
        let path = self.ctx.files.cache.path_of(sha256).ok_or_else(|| MunikoError::from(BoardError::BadFileId))?;
        // Leased while the fetch guard is still held, so cleanup never sees the file unprotected.
        let lease = self.ctx.files.cache.lease(sha256);
        self.ctx.files.cache.touch(sha256);
        publish_if_changed(&self.ctx);
        let refreshed = self.items()?.into_iter().find(|v| v.id == item_id).unwrap_or(view);
        Ok(OpenedFile { path, view: refreshed, lease })
    }

    /// A keeper caches every board file regardless of size, up to the keeper cap.
    pub fn set_keeper(&self, keeper: bool) -> Result<()> {
        self.ctx.replica.store().set_keeper(keeper)?;
        reconcile(&self.ctx)?;
        if let Some(record) = self.ctx.replica.record()? {
            self.ctx.publish_changed(&record.board.id);
        }
        Ok(())
    }

    pub fn is_keeper(&self) -> Result<bool> {
        self.ctx.replica.store().keeper()
    }

    /// Replaces one of this device's own items with a tombstone that syncs to everyone.
    pub fn delete_item(&self, id: &str) -> Result<()> {
        let item = self.ctx.replica.store().get(id)?.ok_or_else(|| MunikoError::from(BoardError::ItemNotFound))?;
        if item.author_id != self.ctx.local_id() || !matches!(item.kind, ItemKind::Text { .. } | ItemKind::File { .. }) {
            return Err(MunikoError::from(BoardError::NotYourItem));
        }
        self.store_local(ItemKind::Tombstone { target_id: id.to_string() })?;
        reconcile(&self.ctx)?;
        publish_if_changed(&self.ctx);
        Ok(())
    }

    pub fn create_board(&self, name: &str, mode: BoardMode, join_code: Option<&str>) -> Result<Board> {
        Membership::new(self.ctx.clone()).create(name, mode, join_code)
    }

    pub async fn join_board(&self, peer_id: &str, code: Option<&str>) -> Result<Board> {
        Membership::new(self.ctx.clone()).join(peer_id, code).await
    }

    pub fn leave_board(&self) -> Result<()> {
        Membership::new(self.ctx.clone()).leave()
    }

    /// Different boards seen on peers while this device is on one; never merged automatically.
    pub fn conflicts(&self) -> Vec<ObservedBoard> {
        match self.ctx.replica.record() {
            Ok(Some(record)) => self.ctx.observed.others(Some(&record.board.id)),
            _ => Vec::new(),
        }
    }

    /// Boards peers offer that this device could join (when it has none).
    pub fn discovered_boards(&self) -> Vec<ObservedBoard> {
        match self.ctx.replica.record() {
            Ok(None) => self.ctx.observed.others(None),
            _ => Vec::new(),
        }
    }

    fn post(&self, kind: ItemKind) -> Result<BoardItemView> {
        let item = self.store_local(kind)?;
        let members = self.ctx.replica.members()?;
        view_of(item, &members, self.ctx.local_id(), &Availability::of(&self.ctx)).ok_or_else(|| MunikoError::from(BoardError::ItemRejected))
    }

    fn store_local(&self, kind: ItemKind) -> Result<BoardItem> {
        let record = self.ctx.replica.require_record()?;
        let item = self.ctx.signer.sign(&record.board.id, kind)?;
        let accepted = self.ctx.replica.accept(vec![item.clone()], unix_now())?;
        if accepted.added.is_empty() {
            return Err(MunikoError::from(BoardError::ItemRejected));
        }
        self.ctx.outbox.push(item.clone());
        self.ctx.publish_changed(&record.board.id);
        Ok(item)
    }
}
