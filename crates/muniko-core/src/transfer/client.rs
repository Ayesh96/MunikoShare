use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use reqwest::{header, Client, RequestBuilder};

use super::digest::describe_file;
use super::manifest::{Manifest, OfferAccepted};
use super::peer_reply::{ensure_accepted, into_transfer_error, transfer_error};
use super::progress::ProgressReporter;
use super::upload::{FileUpload, SharedProgress};
use super::walk::collect_items;
use crate::constants::transfer::*;
use crate::device::DeviceInfo;
use crate::error::Result;
use crate::events::{EventBus, NodeEvent, TransferDirection};
use crate::identity::Identity;
use crate::messages::transfer as msg;
use crate::net::PeerClient;

/// Sends files and folders to a peer over the shared pinned, mutually authenticated client.
pub struct TransferClient {
    identity: Arc<Identity>,
    events: EventBus,
}

impl TransferClient {
    pub fn new(identity: Arc<Identity>, events: EventBus) -> Self {
        Self { identity, events }
    }

    /// Offers the files (folders are walked), waits for the peer, uploads them, and publishes FileSent / TransferFailed.
    pub async fn send_files(&self, peer: &DeviceInfo, paths: &[PathBuf]) -> Result<()> {
        let outcome = self.try_send(peer, paths).await;
        if let Err(error) = &outcome {
            self.events.publish(NodeEvent::TransferFailed { device_id: peer.id.clone(), reason: error.to_string() });
        }
        outcome
    }

    async fn try_send(&self, peer: &DeviceInfo, paths: &[PathBuf]) -> Result<()> {
        let items = collect_items(paths).await?;
        let mut files = Vec::with_capacity(items.len());
        for item in &items {
            files.push(describe_file(&item.path, item.relative.clone()).await?);
        }
        let manifest = Manifest { files };
        let client = PeerClient::new(&self.identity, peer).map_err(into_transfer_error)?;
        let transfer_id = request_offer(&client, &manifest).await?;
        let progress = ProgressReporter::new(self.events.clone(), &transfer_id, &peer.id, TransferDirection::Outgoing, 0, manifest.total_size());
        let progress: SharedProgress = Arc::new(Mutex::new(progress));
        let mut base_bytes = 0;
        for (index, (item, file)) in items.iter().zip(&manifest.files).enumerate() {
            let upload = FileUpload { client: &client, transfer_id: &transfer_id, index, path: &item.path, size: file.size, base_bytes, progress: &progress };
            upload.run().await?;
            base_bytes += file.size;
            if let Ok(mut reporter) = progress.lock() {
                reporter.seek(base_bytes);
                reporter.flush();
            }
            self.events.publish(NodeEvent::FileSent { to_device_id: peer.id.clone(), path: item.path.clone() });
        }
        Ok(())
    }
}

async fn request_offer(client: &PeerClient, manifest: &Manifest) -> Result<String> {
    let payload = serde_json::to_vec(manifest).map_err(|e| transfer_error(msg::SEND_FAILED, e))?;
    let response = client
        .send(|http, base_url| offer_request(http, base_url, payload.clone()))
        .await
        .map_err(into_transfer_error)?;
    let response = ensure_accepted(response).await?;
    let bytes = response.bytes().await.map_err(|e| transfer_error(msg::BAD_OFFER_RESPONSE, e))?;
    let accepted: OfferAccepted = serde_json::from_slice(&bytes).map_err(|e| transfer_error(msg::BAD_OFFER_RESPONSE, e))?;
    Ok(accepted.transfer_id)
}

async fn offer_request(http: Client, base_url: String, payload: Vec<u8>) -> Result<RequestBuilder> {
    Ok(http
        .post(format!("{base_url}{ROUTE_TRANSFERS}"))
        .timeout(OFFER_REQUEST_TIMEOUT)
        .header(header::CONTENT_TYPE, "application/json")
        .body(payload))
}
