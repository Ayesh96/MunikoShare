use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::constants::app::EVENT_CHANNEL_CAPACITY;
use crate::device::DeviceInfo;

/// A file inside a transfer offer, as shown to the receiver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSummary {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransferDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NodeEvent {
    PeerFound { device: DeviceInfo },
    PeerLost { device_id: String },
    /// A peer asked to pair; show `code` so the user can type it on the other device.
    PairingRequested { device: DeviceInfo, code: String },
    PairingCompleted { device_id: String, device_name: String },
    PairingFailed { device_id: String, reason: String },
    DeviceUnpaired { device_id: String },
    /// A trusted peer wants to send files; answer via the node's respond_to_offer.
    IncomingOffer { offer_id: String, from_device_id: String, files: Vec<FileSummary>, total_size: u64 },
    OfferClosed { offer_id: String, accepted: bool },
    TransferProgress { transfer_id: String, device_id: String, direction: TransferDirection, bytes_done: u64, bytes_total: u64 },
    FileReceived { from_device_id: String, path: PathBuf },
    FileSent { to_device_id: String, path: PathBuf },
    TransferFailed { device_id: String, reason: String },
    ClipboardReceived { from_device_id: String, text: String },
    ClipboardSent { to_device_id: String },
    /// An image arrived and was placed on the clipboard.
    ClipboardImageReceived { from_device_id: String, width: u32, height: u32 },
    /// A phone opened the companion page with a valid QR token.
    PhoneConnected { session_id: String, label: String },
    PhoneDisconnected { session_id: String },
    /// The Network Board's content or membership changed; the UI reloads it.
    BoardChanged { board_id: String },
    /// A device was added by IP address rather than discovered.
    PeerAdded { device: DeviceInfo },
}

/// Fan-out channel that services publish to and the UI subscribes to.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<NodeEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn publish(&self, event: NodeEvent) {
        // No subscribers is not an error.
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NodeEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
