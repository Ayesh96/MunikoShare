use std::sync::Arc;

use super::client::BoardClient;
use super::config::BoardConfig;
use super::files::BoardFiles;
use super::observed::ObservedBoards;
use super::outbox::Outbox;
use super::replica::Replica;
use super::signing::ItemSigner;
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::events::{EventBus, NodeEvent};
use crate::messages::errors::PEER_NOT_FOUND;
use crate::util::peers::PeerDirectory;

/// Everything the board's parts share; each part gets an `Arc` of it.
pub struct BoardContext {
    pub signer: ItemSigner,
    pub replica: Arc<Replica>,
    pub client: BoardClient,
    pub events: EventBus,
    pub peers: Arc<dyn PeerDirectory>,
    pub outbox: Arc<Outbox>,
    pub observed: ObservedBoards,
    pub files: BoardFiles,
    pub device_name: String,
    pub config: BoardConfig,
}

impl BoardContext {
    pub fn local_id(&self) -> &str {
        self.signer.device_id()
    }

    pub fn publish_changed(&self, board_id: &str) {
        self.events.publish(NodeEvent::BoardChanged { board_id: board_id.to_string() });
    }

    pub fn find_peer(&self, peer_id: &str) -> Result<DeviceInfo> {
        self.peers.peers().into_iter().find(|peer| peer.id == peer_id).ok_or_else(|| MunikoError::PeerNotFound(format!("{PEER_NOT_FOUND}: {peer_id}")))
    }
}
