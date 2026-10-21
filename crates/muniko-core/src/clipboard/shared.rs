use std::sync::Arc;

use super::access::ClipboardAccess;
use super::history::ClipboardHistory;
use super::recent_hashes::RecentHashes;

/// State the receive route and the watcher must share; build one and pass it to both.
#[derive(Clone)]
pub struct ClipboardShared {
    pub hashes: RecentHashes,
    pub history: ClipboardHistory,
}

impl ClipboardShared {
    pub fn new(access: Arc<dyn ClipboardAccess>) -> Self {
        Self { hashes: RecentHashes::new(), history: ClipboardHistory::new(access) }
    }
}
