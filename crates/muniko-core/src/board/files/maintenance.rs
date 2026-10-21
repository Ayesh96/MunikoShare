//! Keeps the cache honest: drops files of deleted or expired items and evicts down to the cap.

use std::collections::HashSet;

use crate::board::context::BoardContext;
use crate::board::model::ItemKind;
use crate::constants::board::KIND_FILE;
use crate::error::Result;

/// A live file item reduced to what file handling needs.
pub struct FileItem {
    pub id: String,
    pub author_id: String,
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

pub fn file_items(ctx: &BoardContext) -> Result<Vec<FileItem>> {
    let items = ctx.replica.store().by_kind(KIND_FILE)?;
    Ok(items
        .into_iter()
        .filter_map(|item| match item.kind {
            ItemKind::File { name, size, sha256 } => Some(FileItem { id: item.id, author_id: item.author_id, name, size, sha256 }),
            _ => None,
        })
        .collect())
}

/// The cache cap that applies to this device right now.
pub fn current_cap(ctx: &BoardContext) -> Result<u64> {
    Ok(if ctx.replica.store().keeper()? { ctx.config.keeper_cap } else { ctx.config.cache_cap })
}

/// Deletes content nothing references, then evicts least-recently-used foreign files above the cap; true if anything went.
pub fn reconcile(ctx: &BoardContext) -> Result<bool> {
    let items = file_items(ctx)?;
    let referenced: HashSet<&str> = items.iter().map(|item| item.sha256.as_str()).collect();
    let own: HashSet<&str> = items.iter().filter(|item| item.author_id == ctx.local_id()).map(|item| item.sha256.as_str()).collect();
    let cache = &ctx.files.cache;
    let mut removed = false;
    let mut kept = Vec::new();
    for file in cache.scan() {
        if cache.is_busy(&file.sha256) {
            continue;
        }
        if referenced.contains(file.sha256.as_str()) && !file.partial {
            kept.push(file);
        } else if !referenced.contains(file.sha256.as_str()) {
            cache.remove(&file.sha256);
            removed = true;
        }
    }
    Ok(evict(ctx, kept, &own)? || removed)
}

fn evict(ctx: &BoardContext, mut kept: Vec<super::cache::CacheFile>, own: &HashSet<&str>) -> Result<bool> {
    let cap = current_cap(ctx)?;
    let mut usage: u64 = kept.iter().map(|file| file.size).sum();
    kept.retain(|file| !own.contains(file.sha256.as_str()));
    kept.sort_by_key(|file| file.used);
    let mut removed = false;
    for file in kept {
        if usage <= cap {
            break;
        }
        ctx.files.cache.remove(&file.sha256);
        usage = usage.saturating_sub(file.size);
        removed = true;
    }
    Ok(removed)
}
