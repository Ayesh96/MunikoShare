use std::sync::Arc;

use futures_util::future::join_all;

use super::client::ClipboardClient;
use super::content::ClipboardContent;
use crate::constants::clipboard::SEND_TIMEOUT;
use crate::device::DeviceInfo;
use crate::events::{EventBus, NodeEvent};
use crate::identity::Identity;
use crate::messages::clipboard as msg;

/// Fans one clipboard change out to several peers concurrently.
pub struct AutoSender {
    client: ClipboardClient,
    events: EventBus,
}

impl AutoSender {
    pub fn new(identity: Arc<Identity>, events: EventBus) -> Self {
        Self { client: ClipboardClient::new(identity), events }
    }

    /// Failures are logged only, so an offline peer never floods the UI with events.
    pub async fn send_all(&self, targets: &[DeviceInfo], content: &ClipboardContent) {
        join_all(targets.iter().map(|peer| self.send_one(peer, content))).await;
    }

    async fn send_one(&self, peer: &DeviceInfo, content: &ClipboardContent) {
        match tokio::time::timeout(SEND_TIMEOUT, self.client.push_content(peer, content)).await {
            Ok(Ok(())) => self.events.publish(NodeEvent::ClipboardSent { to_device_id: peer.id.clone() }),
            Ok(Err(error)) => tracing::warn!(to = %peer.id, %error, "{}", msg::AUTO_SEND_FAILED),
            Err(_) => tracing::warn!(to = %peer.id, "{}", msg::AUTO_SEND_TIMED_OUT),
        }
    }
}
