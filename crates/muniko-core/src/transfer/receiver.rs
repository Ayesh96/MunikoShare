//! Streams one accepted file into its partial file, verifies it against the manifest, and publishes the result.

use std::path::PathBuf;

use axum::body::Body;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use super::incoming::FileTicket;
use super::partial::PartialStore;
use super::progress::ProgressReporter;
use super::rejection::Rejection;
use crate::constants::transfer::*;
use crate::events::{EventBus, NodeEvent, TransferDirection};
use crate::messages::transfer as msg;
use crate::util::fs::claim_unique_path;

pub struct FileReceiver {
    download_dir: PathBuf,
    partials: PartialStore,
    events: EventBus,
}

impl FileReceiver {
    pub fn new(download_dir: PathBuf, events: EventBus) -> Self {
        Self { partials: PartialStore::new(download_dir.clone()), download_dir, events }
    }

    /// Appends the body at `offset` to the file's partial and finalizes it once complete.
    pub async fn receive(&self, ticket: &FileTicket, offset: u64, body: Body) -> Result<PathBuf, Rejection> {
        match self.store(ticket, offset, body).await {
            Ok(path) => {
                tracing::info!(path = %path.display(), "{}", msg::FILE_RECEIVED);
                self.events.publish(NodeEvent::FileReceived { from_device_id: ticket.device_id.clone(), path: path.clone() });
                Ok(path)
            }
            Err(rejection) => {
                tracing::warn!(?rejection, "{}", msg::REQUEST_REJECTED);
                Err(rejection)
            }
        }
    }

    async fn store(&self, ticket: &FileTicket, offset: u64, body: Body) -> Result<PathBuf, Rejection> {
        let (file, hasher) = self.partials.open_at(&ticket.transfer_id, ticket.index, offset).await?;
        self.write_verified(ticket, offset, file, hasher, body).await?;
        let final_path = self.claim_destination(ticket).await?;
        let partial = self.partials.path(&ticket.transfer_id, ticket.index);
        fs::rename(partial, &final_path).await.map_err(|_| Rejection::Internal(msg::STORE_FAILED))?;
        Ok(final_path)
    }

    // An interrupted or short upload keeps its partial so the sender can resume; corrupt data is dropped.
    async fn write_verified(&self, ticket: &FileTicket, offset: u64, mut file: File, mut hasher: Sha256, body: Body) -> Result<(), Rejection> {
        let meta = &ticket.file;
        let mut progress =
            ProgressReporter::new(self.events.clone(), &ticket.transfer_id, &ticket.device_id, TransferDirection::Incoming, ticket.base_bytes + offset, ticket.total_bytes);
        let store_failed = |_| Rejection::Internal(msg::STORE_FAILED);
        let mut written = offset;
        let mut stream = body.into_data_stream();
        while let Some(chunk) = stream.next().await {
            let Ok(chunk) = chunk else {
                let _ = file.flush().await;
                return Err(Rejection::BadRequest(msg::UPLOAD_INTERRUPTED));
            };
            written += chunk.len() as u64;
            if written > meta.size {
                self.partials.discard(&ticket.transfer_id, ticket.index).await;
                return Err(Rejection::BadRequest(msg::SIZE_MISMATCH));
            }
            hasher.update(&chunk);
            file.write_all(&chunk).await.map_err(store_failed)?;
            if progress.add(chunk.len() as u64) {
                ticket.activity.touch();
            }
        }
        file.flush().await.map_err(store_failed)?;
        if written != meta.size {
            return Err(Rejection::BadRequest(msg::UPLOAD_INCOMPLETE));
        }
        if hex::encode(hasher.finalize()) != meta.sha256 {
            self.partials.discard(&ticket.transfer_id, ticket.index).await;
            return Err(Rejection::ChecksumMismatch);
        }
        progress.flush();
        Ok(())
    }

    // Files in a sent folder go under that transfer's resolved root folder.
    async fn claim_destination(&self, ticket: &FileTicket) -> Result<PathBuf, Rejection> {
        let parts: Vec<&str> = ticket.file.path.split(PATH_SEPARATOR).collect();
        let (name, folders) = parts.split_last().ok_or(Rejection::Internal(msg::STORE_FAILED))?;
        let mut dir = self.download_dir.clone();
        if let Some((top, nested)) = folders.split_first() {
            dir = ticket.roots.resolve(top).await?;
            dir.extend(nested);
            fs::create_dir_all(&dir).await.map_err(|_| Rejection::Internal(msg::STORE_FAILED))?;
        }
        claim_unique_path(&dir, name).await.map_err(|_| Rejection::Internal(msg::STORE_FAILED))
    }
}
