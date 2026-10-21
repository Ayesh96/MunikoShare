//! Which files this device holds or could fetch right now, and when that is worth telling the UI.

use std::collections::HashSet;

use super::maintenance::file_items;
use crate::board::context::BoardContext;
use crate::board::model::ItemKind;
use crate::util::lock::lock;

/// Snapshot of held and online-available file hashes.
pub struct Availability {
    held: HashSet<String>,
    online: HashSet<String>,
}

impl Availability {
    pub fn of(ctx: &BoardContext) -> Self {
        let held = ctx.files.cache.held().into_iter().collect();
        let online = ctx.files.holders.online_hashes(&ctx.peers.peers());
        Self { held, online }
    }

    /// (held locally, available from this device or an online peer); text is always both.
    pub fn status(&self, kind: &ItemKind) -> (bool, bool) {
        match kind {
            ItemKind::File { sha256, .. } => self.of_hash(sha256),
            _ => (true, true),
        }
    }

    pub fn of_hash(&self, sha256: &str) -> (bool, bool) {
        let held = self.held.contains(sha256);
        (held, held || self.online.contains(sha256))
    }
}

/// Publishes BoardChanged only when some file's availability differs from the last time.
pub fn publish_if_changed(ctx: &BoardContext) {
    let Ok(items) = file_items(ctx) else { return };
    let Ok(Some(record)) = ctx.replica.record() else { return };
    let snapshot = Availability::of(ctx);
    let mut current: Vec<(String, bool, bool)> = items
        .into_iter()
        .map(|item| {
            let (held, available) = snapshot.of_hash(&item.sha256);
            (item.id, held, available)
        })
        .collect();
    current.sort();
    let mut last = lock(&ctx.files.last_availability);
    if *last != current {
        *last = current;
        drop(last);
        ctx.publish_changed(&record.board.id);
    }
}
