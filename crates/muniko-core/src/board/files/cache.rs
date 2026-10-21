//! The content-addressed cache: one file per SHA-256 under the cache dir.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use tokio::fs::File;
use tokio::sync::OwnedMutexGuard;

use super::lease::{FileLease, Leases};
use crate::constants::board::{INGEST_PREFIX, PART_SUFFIX};
use crate::board::BoardError;
use crate::error::{MunikoError, Result};
use crate::messages::board as msg;
use crate::util::fs::ensure_dir;
use crate::util::hash::{copy_hashed, is_sha256_hex};
use crate::util::slot_locks::SlotLocks;

/// One file found in the cache directory.
pub struct CacheFile {
    pub sha256: String,
    pub size: u64,
    pub used: SystemTime,
    /// A download still in progress (`<sha>.part`).
    pub partial: bool,
}

/// Result of copying a local file into the cache.
pub struct Ingested {
    pub sha256: String,
    pub size: u64,
}

pub struct FileCache {
    dir: PathBuf,
    locks: SlotLocks,
    leases: Arc<Leases>,
}

fn cache_error(error: impl std::fmt::Display) -> MunikoError {
    MunikoError::from(BoardError::storage(msg::CACHE_FAILED, error))
}

impl FileCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir, locks: SlotLocks::default(), leases: Arc::default() }
    }

    pub async fn ensure_dir(&self) -> Result<()> {
        ensure_dir(&self.dir).await
    }

    pub fn path_of(&self, sha256: &str) -> Option<PathBuf> {
        is_sha256_hex(sha256).then(|| self.dir.join(sha256))
    }

    pub fn part_path(&self, sha256: &str) -> PathBuf {
        self.dir.join(format!("{sha256}{PART_SUFFIX}"))
    }

    pub fn has(&self, sha256: &str) -> bool {
        self.path_of(sha256).is_some_and(|path| path.is_file())
    }

    /// Serializes work on one hash (fetching, ingesting) and keeps cleanup away from it.
    pub async fn lock(&self, sha256: &str) -> OwnedMutexGuard<()> {
        self.locks.lock(sha256).await
    }

    /// True while the hash is being worked on or leased; cleanup must skip it.
    pub fn is_busy(&self, sha256: &str) -> bool {
        self.locks.is_locked(sha256) || self.leases.is_leased(sha256)
    }

    /// Pins the file so cleanup and eviction cannot delete it until the lease drops.
    pub fn lease(&self, sha256: &str) -> FileLease {
        FileLease::new(&self.leases, sha256)
    }

    /// Every cache file, complete or partial; unreadable entries are skipped.
    pub fn scan(&self) -> Vec<CacheFile> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else { return Vec::new() };
        entries.flatten().filter_map(|entry| Self::describe(&entry)).collect()
    }

    fn describe(entry: &std::fs::DirEntry) -> Option<CacheFile> {
        let name = entry.file_name().into_string().ok()?;
        let (sha256, partial) = match name.strip_suffix(PART_SUFFIX) {
            Some(stem) => (stem.to_string(), true),
            None => (name, false),
        };
        let metadata = entry.metadata().ok().filter(|m| m.is_file() && is_sha256_hex(&sha256))?;
        Some(CacheFile { sha256, size: metadata.len(), used: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH), partial })
    }

    /// Hashes of the complete files this device can serve.
    pub fn held(&self) -> Vec<String> {
        self.scan().into_iter().filter(|file| !file.partial).map(|file| file.sha256).collect()
    }

    /// Marks a file as just used so least-recently-used eviction spares it.
    pub fn touch(&self, sha256: &str) {
        let Some(path) = self.path_of(sha256) else { return };
        if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) {
            let _ = file.set_modified(SystemTime::now());
        }
    }

    pub fn remove(&self, sha256: &str) {
        if self.leases.is_leased(sha256) {
            return;
        }
        if let Some(path) = self.path_of(sha256) {
            let _ = std::fs::remove_file(path);
        }
        let _ = std::fs::remove_file(self.part_path(sha256));
    }

    /// Streams a local file into the cache while hashing it; the guard keeps the entry from cleanup until dropped.
    pub async fn ingest(&self, source: &Path) -> Result<(Ingested, OwnedMutexGuard<()>)> {
        self.ensure_dir().await?;
        let temp = self.dir.join(format!("{INGEST_PREFIX}{}{PART_SUFFIX}", uuid::Uuid::new_v4()));
        let copied = Self::copy_into(source, &temp).await;
        let (sha256, size) = match copied {
            Ok(done) => done,
            Err(error) => {
                let _ = tokio::fs::remove_file(&temp).await;
                return Err(cache_error(error));
            }
        };
        let guard = self.lock(&sha256).await;
        let target = self.dir.join(&sha256);
        if target.is_file() {
            let _ = tokio::fs::remove_file(&temp).await;
            self.touch(&sha256);
        } else if let Err(error) = tokio::fs::rename(&temp, &target).await {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(cache_error(error));
        }
        Ok((Ingested { sha256, size }, guard))
    }

    async fn copy_into(source: &Path, temp: &Path) -> std::io::Result<(String, u64)> {
        let mut reader = File::open(source).await?;
        let mut writer = File::create(temp).await?;
        copy_hashed(&mut reader, &mut writer).await
    }
}
