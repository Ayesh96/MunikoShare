//! Fetches the files this device should hold without being asked.

use super::fetch::ensure_cached;
use super::maintenance::{current_cap, file_items};
use crate::board::context::BoardContext;
use crate::error::Result;

/// Caches missing files that are small enough (any size on a keeper) while the cap has room; true if one arrived.
pub async fn cache_wanted(ctx: &BoardContext) -> Result<bool> {
    let keeper = ctx.replica.store().keeper()?;
    let cap = current_cap(ctx)?;
    let mut usage: u64 = ctx.files.cache.scan().iter().filter(|file| !file.partial).map(|file| file.size).sum();
    let mut fetched = false;
    for item in file_items(ctx)? {
        let wanted = keeper || item.size <= ctx.config.auto_cache_limit;
        // Eviction only trims what is over the cap; auto-caching never pushes it over, which would thrash.
        if ctx.files.cache.has(&item.sha256) || !wanted || usage + item.size > cap {
            continue;
        }
        let holders = ctx.files.holders.of(&item.sha256, &ctx.peers.peers());
        if holders.is_empty() {
            continue;
        }
        if ensure_cached(ctx, &item.sha256, item.size, &holders).await.is_ok() {
            usage += item.size;
            fetched = true;
        }
    }
    Ok(fetched)
}
