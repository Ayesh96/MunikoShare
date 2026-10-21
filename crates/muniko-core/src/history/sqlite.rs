use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection};

use super::{HistoryEntry, HistoryKind, HistoryStore, NewHistoryEntry};
use crate::constants::history::*;
use crate::error::{MunikoError, Result};
use crate::messages::history as msg;

fn db_error(context: &str, detail: impl std::fmt::Display) -> MunikoError {
    MunikoError::Io(std::io::Error::other(format!("{context}: {detail}")))
}

fn query_error(error: rusqlite::Error) -> MunikoError {
    db_error(msg::QUERY_FAILED, error)
}

/// History persisted in the `history` table of a SQLite file.
pub struct SqliteHistoryStore {
    connection: Mutex<Connection>,
}

impl SqliteHistoryStore {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path).map_err(|e| db_error(msg::OPEN_FAILED, e))?;
        connection.execute(CREATE_TABLE_SQL, []).map_err(|e| db_error(msg::OPEN_FAILED, e))?;
        Ok(Self { connection: Mutex::new(connection) })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| db_error(msg::QUERY_FAILED, msg::LOCK_POISONED))
    }
}

fn read_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
    let kind: String = row.get(2)?;
    let kind = HistoryKind::parse(&kind)
        .ok_or_else(|| rusqlite::Error::InvalidColumnType(2, msg::UNKNOWN_KIND.into(), rusqlite::types::Type::Text))?;
    Ok(HistoryEntry { id: row.get(0)?, timestamp: row.get::<_, i64>(1)? as u64, kind, device_id: row.get(3)?, detail: row.get(4)? })
}

impl HistoryStore for SqliteHistoryStore {
    fn record(&self, entry: NewHistoryEntry) -> Result<()> {
        let timestamp = i64::try_from(entry.timestamp).unwrap_or(i64::MAX);
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(query_error)?;
        tx.execute(INSERT_SQL, params![timestamp, entry.kind.as_str(), entry.device_id, entry.detail]).map_err(query_error)?;
        tx.execute(PRUNE_SQL, params![MAX_HISTORY_ENTRIES as i64]).map_err(query_error)?;
        tx.commit().map_err(query_error)
    }

    fn recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(SELECT_RECENT_SQL).map_err(query_error)?;
        let rows = statement.query_map(params![limit as i64], read_entry).map_err(query_error)?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(query_error)
    }

    fn clear(&self) -> Result<()> {
        self.lock()?.execute(CLEAR_SQL, []).map_err(query_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn open(dir: &TempDir) -> SqliteHistoryStore {
        SqliteHistoryStore::open(&dir.path().join("muniko.db")).unwrap()
    }

    fn entry(detail: &str) -> NewHistoryEntry {
        NewHistoryEntry { timestamp: 42, kind: HistoryKind::FileSent, device_id: "dev".into(), detail: detail.into() }
    }

    #[test]
    fn round_trip_and_newest_first_with_limit() {
        let dir = TempDir::new().unwrap();
        let store = open(&dir);
        for name in ["a", "b", "c"] {
            store.record(entry(name)).unwrap();
        }
        let all = store.recent(10).unwrap();
        let details: Vec<_> = all.iter().map(|e| e.detail.as_str()).collect();
        assert_eq!(details, ["c", "b", "a"]);
        assert_eq!((all[0].timestamp, all[0].kind, all[0].device_id.as_str()), (42, HistoryKind::FileSent, "dev"));
        assert_eq!(store.recent(2).unwrap().len(), 2);
    }

    #[test]
    fn prunes_to_cap_keeping_newest() {
        let dir = TempDir::new().unwrap();
        let store = open(&dir);
        for i in 0..MAX_HISTORY_ENTRIES + 5 {
            store.record(entry(&i.to_string())).unwrap();
        }
        let all = store.recent(usize::MAX >> 1).unwrap();
        assert_eq!(all.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(all[0].detail, (MAX_HISTORY_ENTRIES + 4).to_string());
    }

    #[test]
    fn clear_empties_and_data_survives_reopen() {
        let dir = TempDir::new().unwrap();
        open(&dir).record(entry("x")).unwrap();
        let store = open(&dir);
        assert_eq!(store.recent(5).unwrap().len(), 1);
        store.clear().unwrap();
        assert!(store.recent(5).unwrap().is_empty());
    }

    #[test]
    fn shares_file_with_trust_store() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("muniko.db");
        let _trust = crate::trust::SqliteTrustStore::open(&path).unwrap();
        let store = SqliteHistoryStore::open(&path).unwrap();
        store.record(entry("x")).unwrap();
        assert_eq!(store.recent(1).unwrap().len(), 1);
    }
}
