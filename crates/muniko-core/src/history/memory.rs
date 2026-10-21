use std::sync::Mutex;

use super::{HistoryEntry, HistoryStore, NewHistoryEntry};
use crate::constants::history::MAX_HISTORY_ENTRIES;
use crate::error::{MunikoError, Result};
use crate::messages::history as msg;

#[derive(Default)]
struct Rows {
    next_id: i64,
    entries: Vec<HistoryEntry>,
}

/// In-memory store for tests.
#[derive(Default)]
pub struct MemoryHistoryStore {
    rows: Mutex<Rows>,
}

impl MemoryHistoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Rows>> {
        self.rows.lock().map_err(|_| MunikoError::Io(std::io::Error::other(msg::LOCK_POISONED)))
    }
}

impl HistoryStore for MemoryHistoryStore {
    fn record(&self, entry: NewHistoryEntry) -> Result<()> {
        let mut rows = self.lock()?;
        rows.next_id += 1;
        let id = rows.next_id;
        rows.entries.push(HistoryEntry { id, timestamp: entry.timestamp, kind: entry.kind, device_id: entry.device_id, detail: entry.detail });
        if rows.entries.len() > MAX_HISTORY_ENTRIES {
            let excess = rows.entries.len() - MAX_HISTORY_ENTRIES;
            rows.entries.drain(..excess);
        }
        Ok(())
    }

    fn recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        Ok(self.lock()?.entries.iter().rev().take(limit).cloned().collect())
    }

    fn clear(&self) -> Result<()> {
        self.lock()?.entries.clear();
        Ok(())
    }
}
