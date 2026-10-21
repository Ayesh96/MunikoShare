use std::sync::{Arc, RwLock};

use crate::device::DeviceInfo;

/// Read-only view of currently reachable peers; the node implements it over discovery.
pub trait PeerDirectory: Send + Sync {
    fn peers(&self) -> Vec<DeviceInfo>;
}

/// Forwards to whichever directory is attached; empty until discovery starts.
#[derive(Default)]
pub struct LivePeers {
    source: RwLock<Option<Arc<dyn PeerDirectory>>>,
}

impl LivePeers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attach(&self, directory: Arc<dyn PeerDirectory>) {
        if let Ok(mut source) = self.source.write() {
            *source = Some(directory);
        }
    }

    pub fn detach(&self) {
        if let Ok(mut source) = self.source.write() {
            *source = None;
        }
    }
}

impl PeerDirectory for LivePeers {
    fn peers(&self) -> Vec<DeviceInfo> {
        let source = self.source.read().ok().and_then(|source| source.clone());
        source.map(|directory| directory.peers()).unwrap_or_default()
    }
}
