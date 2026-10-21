use std::sync::Arc;

use super::access::ClipboardAccess;
use super::client::ClipboardClient;
use super::content::ClipboardContent;
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::clipboard as msg;

/// Facade for sending clipboard content to peers.
pub struct ClipboardService {
    client: ClipboardClient,
    access: Arc<dyn ClipboardAccess>,
}

impl ClipboardService {
    pub fn new(identity: Arc<Identity>, access: Arc<dyn ClipboardAccess>) -> Self {
        Self { client: ClipboardClient::new(identity), access }
    }

    pub async fn push_text(&self, peer: &DeviceInfo, text: &str) -> Result<()> {
        self.client.push_text(peer, text).await
    }

    pub async fn push_content(&self, peer: &DeviceInfo, content: &ClipboardContent) -> Result<()> {
        self.client.push_content(peer, content).await
    }

    /// Sends whatever text or image is on the local clipboard right now.
    pub async fn push_current(&self, peer: &DeviceInfo) -> Result<()> {
        let access = self.access.clone();
        let content = tokio::task::spawn_blocking(move || access.read_content())
            .await
            .map_err(|e| MunikoError::InvalidState(format!("{}: {e}", msg::TASK_FAILED)))??;
        match content {
            Some(content) => self.push_content(peer, &content).await,
            None => Err(MunikoError::InvalidState(msg::LOCAL_CLIPBOARD_EMPTY.to_string())),
        }
    }
}
