//! The `.part` files of accepted transfers; kept after an interruption so uploads can resume.

use std::path::PathBuf;

use sha2::Sha256;
use tokio::fs::{self, File, OpenOptions};

use super::rejection::Rejection;
use crate::constants::transfer::{TEMP_FILE_PREFIX, TEMP_FILE_SUFFIX};
use crate::messages::transfer as msg;
use crate::util::hash::hash_up_to;

#[derive(Clone)]
pub struct PartialStore {
    dir: PathBuf,
}

impl PartialStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn path(&self, transfer_id: &str, index: usize) -> PathBuf {
        self.dir.join(format!("{TEMP_FILE_PREFIX}{transfer_id}-{index}{TEMP_FILE_SUFFIX}"))
    }

    /// Bytes stored so far; 0 when nothing arrived yet.
    pub async fn len(&self, transfer_id: &str, index: usize) -> u64 {
        fs::metadata(self.path(transfer_id, index)).await.map(|m| m.len()).unwrap_or(0)
    }

    /// Opens for appending at `offset` (must equal the stored length) with the stored bytes already hashed.
    pub async fn open_at(&self, transfer_id: &str, index: usize, offset: u64) -> Result<(File, Sha256), Rejection> {
        if self.len(transfer_id, index).await != offset {
            return Err(Rejection::Conflict(msg::OFFSET_MISMATCH));
        }
        let path = self.path(transfer_id, index);
        let store_failed = |_| Rejection::Internal(msg::STORE_FAILED);
        let hasher = match offset {
            0 => Sha256::default(),
            _ => hash_up_to(&mut File::open(&path).await.map_err(store_failed)?, offset).await.map_err(store_failed)?.0,
        };
        let file = OpenOptions::new().create(true).append(true).open(&path).await.map_err(store_failed)?;
        Ok((file, hasher))
    }

    pub async fn discard(&self, transfer_id: &str, index: usize) {
        let _ = fs::remove_file(self.path(transfer_id, index)).await;
    }

    /// Synchronous so the idle sweep can call it while holding the transfer state.
    pub fn discard_transfer(&self, transfer_id: &str, file_count: usize) {
        for index in 0..file_count {
            let _ = std::fs::remove_file(self.path(transfer_id, index));
        }
    }
}
