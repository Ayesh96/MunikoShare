//! Downloads one file into the cache from any holder: resumable `.part`, hash-verified, next holder on failure.

use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::OwnedMutexGuard;

use crate::board::context::BoardContext;
use crate::device::DeviceInfo;
use crate::board::BoardError;
use crate::error::{MunikoError, Result};
use crate::messages::board as msg;
use crate::util::hash::hash_up_to;

fn io_fail(error: std::io::Error) -> MunikoError {
    MunikoError::from(BoardError::storage(msg::CACHE_FAILED, error))
}

/// Makes sure `sha256` is cached, trying each holder in turn; the guard keeps cleanup away until dropped.
pub async fn ensure_cached(ctx: &BoardContext, sha256: &str, size: u64, holders: &[DeviceInfo]) -> Result<OwnedMutexGuard<()>> {
    let guard = ctx.files.cache.lock(sha256).await;
    if ctx.files.cache.has(sha256) {
        return Ok(guard);
    }
    if holders.is_empty() {
        return Err(MunikoError::from(BoardError::NoHolder));
    }
    for peer in holders {
        match download_from(ctx, peer, sha256, size).await {
            Ok(()) => return Ok(guard),
            Err(error) => tracing::debug!(peer = %peer.id, %error, "{}", msg::LOG_FETCH_FAILED),
        }
    }
    Err(MunikoError::from(BoardError::FetchFailed))
}

async fn download_from(ctx: &BoardContext, peer: &DeviceInfo, sha256: &str, size: u64) -> Result<()> {
    let cache = &ctx.files.cache;
    cache.ensure_dir().await?;
    let part = cache.part_path(sha256);
    let stored = fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
    let mut offset = if stored > size { 0 } else { stored };
    let mut hasher = Sha256::new();
    if offset < size || size == 0 {
        let (response, resumed) = ctx.client.open_file(peer, sha256, offset).await?;
        if !resumed {
            offset = 0;
        }
        let mut file = open_part(&part, offset).await?;
        if offset > 0 {
            hasher = hash_up_to(&mut File::open(&part).await.map_err(io_fail)?, offset).await.map_err(io_fail)?.0;
        }
        write_stream(response, &mut file, &mut hasher, offset, size).await.inspect_err(|_| discard_if_oversized(&part, size))?;
    } else {
        hasher = hash_up_to(&mut File::open(&part).await.map_err(io_fail)?, size).await.map_err(io_fail)?.0;
    }
    publish(ctx, &part, sha256, hasher).await
}

// A holder that sends more than the item's size is lying; its bytes must not seed a resume.
fn discard_if_oversized(part: &std::path::Path, size: u64) {
    if std::fs::metadata(part).is_ok_and(|m| m.len() > size) {
        let _ = std::fs::remove_file(part);
    }
}

/// Appends at `offset`, or starts the file over when the peer sent the whole body.
async fn open_part(part: &std::path::Path, offset: u64) -> Result<File> {
    let mut options = OpenOptions::new();
    options.create(true);
    if offset == 0 {
        options.write(true).truncate(true);
    } else {
        options.append(true);
    }
    options.open(part).await.map_err(io_fail)
}

async fn write_stream(response: reqwest::Response, file: &mut File, hasher: &mut Sha256, offset: u64, size: u64) -> Result<()> {
    let mut written = offset;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let Ok(chunk) = chunk else {
            let _ = file.flush().await;
            return Err(MunikoError::from(BoardError::FetchInterrupted));
        };
        written += chunk.len() as u64;
        hasher.update(&chunk);
        file.write_all(&chunk).await.map_err(io_fail)?;
        if written > size {
            return Err(MunikoError::from(BoardError::SizeMismatch));
        }
    }
    file.flush().await.map_err(io_fail)?;
    if written == size {
        Ok(())
    } else {
        Err(MunikoError::from(BoardError::FetchInterrupted))
    }
}

/// Moves a complete `.part` into place when its hash matches; a lying holder's bytes are dropped.
async fn publish(ctx: &BoardContext, part: &std::path::Path, sha256: &str, hasher: Sha256) -> Result<()> {
    if hex::encode(hasher.finalize()) != sha256 {
        let _ = fs::remove_file(part).await;
        return Err(MunikoError::from(BoardError::HashMismatch));
    }
    let target = ctx.files.cache.path_of(sha256).ok_or_else(|| MunikoError::from(BoardError::BadFileId))?;
    fs::rename(part, target).await.map_err(io_fail)
}
