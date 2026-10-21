//! Board file content: cache, serving, holder tracking, fetching and cleanup.

mod auto;
mod availability;
mod cache;
mod export;
mod fetch;
mod holders;
mod lease;
mod maintenance;
mod serve;
mod worker;

use std::sync::Arc;

pub use availability::{publish_if_changed, Availability};
pub use cache::FileCache;
pub use export::export_to_downloads;
pub use fetch::ensure_cached;
pub use lease::FileLease;
pub use maintenance::reconcile;
pub use serve::serve_file;
pub use worker::FileWorker;

use holders::Holders;

/// Per-device state of board file content, shared through the board context.
pub struct BoardFiles {
    pub cache: Arc<FileCache>,
    pub holders: Holders,
    pub(self) last_availability: std::sync::Mutex<Vec<(String, bool, bool)>>,
}

impl BoardFiles {
    pub fn new(cache: Arc<FileCache>) -> Self {
        Self { cache, holders: Holders::default(), last_availability: Default::default() }
    }
}
