//! Copies a cached board file into the user's download folder under a unique name.

use std::path::PathBuf;

use super::fetch::ensure_cached;
use super::maintenance::file_items;
use crate::board::context::BoardContext;
use crate::board::BoardError;
use crate::error::{MunikoError, Result};
use crate::messages::board as msg;
use crate::util::fs::{claim_unique_path, ensure_dir};
use crate::util::sanitize::sanitize_file_name;

/// Fetches the item's file into the cache if needed, then exports it; returns the new path.
pub async fn export_to_downloads(ctx: &BoardContext, item_id: &str) -> Result<PathBuf> {
    let item = file_items(ctx)?.into_iter().find(|item| item.id == item_id).ok_or_else(|| MunikoError::from(BoardError::NotAFileItem))?;
    let holders = ctx.files.holders.of(&item.sha256, &ctx.peers.peers());
    let _guard = ensure_cached(ctx, &item.sha256, item.size, &holders).await?;
    let source = ctx.files.cache.path_of(&item.sha256).ok_or_else(|| MunikoError::from(BoardError::BadFileId))?;
    let name = sanitize_file_name(&item.name).ok_or_else(|| MunikoError::from(BoardError::InvalidName))?;
    ensure_dir(&ctx.config.download_dir).await?;
    let target = claim_unique_path(&ctx.config.download_dir, &name).await?;
    if let Err(error) = tokio::fs::copy(&source, &target).await {
        let _ = tokio::fs::remove_file(&target).await;
        return Err(MunikoError::from(BoardError::storage(msg::EXPORT_FAILED, error)));
    }
    ctx.files.cache.touch(&item.sha256);
    super::availability::publish_if_changed(ctx);
    Ok(target)
}
