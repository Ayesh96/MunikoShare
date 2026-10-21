use std::sync::Arc;

use reqwest::header;

use super::content::ClipboardContent;
use super::wire::ClipboardPayload;
use crate::constants::clipboard::{JSON_CONTENT_TYPE, ROUTE_CLIPBOARD};
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::clipboard as msg;
use crate::net::PeerClient;

/// Pushes text or images to a peer over the shared pinned, mutually authenticated client.
pub struct ClipboardClient {
    identity: Arc<Identity>,
}

impl ClipboardClient {
    pub fn new(identity: Arc<Identity>) -> Self {
        Self { identity }
    }

    pub async fn push_text(&self, peer: &DeviceInfo, text: &str) -> Result<()> {
        self.push_content(peer, &ClipboardContent::Text(text.to_string())).await
    }

    pub async fn push_content(&self, peer: &DeviceInfo, content: &ClipboardContent) -> Result<()> {
        if let Some(problem) = content.problem() {
            return Err(MunikoError::InvalidState(problem.message().to_string()));
        }
        let client = PeerClient::new(&self.identity, peer)?;
        let body = serde_json::to_vec(&ClipboardPayload::from_content(content))
            .map_err(|e| MunikoError::InvalidState(format!("{}: {e}", msg::SEND_FAILED)))?;
        let response = client
            .send(|http, base_url| {
                let request = http
                    .post(format!("{base_url}{ROUTE_CLIPBOARD}"))
                    .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
                    .body(body.clone());
                async move { Ok(request) }
            })
            .await
            .map_err(|e| MunikoError::Network(format!("{}: {e}", msg::SEND_FAILED)))?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let reason = response.text().await.unwrap_or_default();
        Err(MunikoError::Network(format!("{}: {status} {reason}", msg::PEER_REJECTED)))
    }
}
