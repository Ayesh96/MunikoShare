//! Transfer history: a persistent log of sent and received items.

mod entry;
mod memory;
mod recorder;
mod sqlite;

pub use entry::{clipboard_preview, HistoryEntry, HistoryKind, NewHistoryEntry};
pub use memory::MemoryHistoryStore;
pub use recorder::HistoryRecorder;
pub use sqlite::SqliteHistoryStore;

use crate::error::Result;

/// Persistent activity log; implementations keep at most MAX_HISTORY_ENTRIES rows.
pub trait HistoryStore: Send + Sync {
    fn record(&self, entry: NewHistoryEntry) -> Result<()>;
    /// Newest first.
    fn recent(&self, limit: usize) -> Result<Vec<HistoryEntry>>;
    fn clear(&self) -> Result<()>;
}
