use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection};

use super::{TrustStore, TrustedDevice};
use crate::constants::trust::*;
use crate::error::{MunikoError, Result};
use crate::messages::trust as msg;

fn db_error(context: &str, detail: impl std::fmt::Display) -> MunikoError {
    MunikoError::Io(std::io::Error::other(format!("{context}: {detail}")))
}

/// Trusted devices persisted in a SQLite file.
pub struct SqliteTrustStore {
    connection: Mutex<Connection>,
}

impl SqliteTrustStore {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path).map_err(|e| db_error(msg::OPEN_FAILED, e))?;
        connection.execute(CREATE_TABLE_SQL, []).map_err(|e| db_error(msg::OPEN_FAILED, e))?;
        connection.execute(CREATE_SETTINGS_TABLE_SQL, []).map_err(|e| db_error(msg::OPEN_FAILED, e))?;
        add_column_if_missing(&connection, ADD_CLIPBOARD_SYNC_COLUMN_SQL)?;
        Ok(Self { connection: Mutex::new(connection) })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| db_error(msg::QUERY_FAILED, msg::LOCK_POISONED))
    }
}

fn query_error(error: rusqlite::Error) -> MunikoError {
    db_error(msg::QUERY_FAILED, error)
}

impl TrustStore for SqliteTrustStore {
    fn is_trusted(&self, device_id: &str) -> bool {
        let Ok(connection) = self.lock() else { return false };
        connection.query_row(EXISTS_SQL, params![device_id], |_| Ok(())).is_ok()
    }

    fn list(&self) -> Result<Vec<TrustedDevice>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(SELECT_ALL_SQL).map_err(query_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(TrustedDevice { id: row.get(0)?, name: row.get(1)?, os: row.get(2)?, paired_at: row.get::<_, i64>(3)? as u64 })
            })
            .map_err(query_error)?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(query_error)
    }

    fn add(&self, device: TrustedDevice) -> Result<()> {
        let paired_at = i64::try_from(device.paired_at).map_err(|_| db_error(msg::QUERY_FAILED, msg::PAIRED_AT_OUT_OF_RANGE))?;
        let connection = self.lock()?;
        connection.execute(UPSERT_SQL, params![device.id, device.name, device.os, paired_at]).map_err(query_error)?;
        Ok(())
    }

    fn remove(&self, device_id: &str) -> Result<bool> {
        let connection = self.lock()?;
        connection.execute(DELETE_SETTINGS_SQL, params![device_id]).map_err(query_error)?;
        Ok(connection.execute(DELETE_SQL, params![device_id]).map_err(query_error)? > 0)
    }

    fn auto_accept(&self, device_id: &str) -> bool {
        self.read_flag(SELECT_AUTO_ACCEPT_SQL, device_id)
    }

    fn set_auto_accept(&self, device_id: &str, enabled: bool) -> Result<()> {
        self.write_flag(UPSERT_AUTO_ACCEPT_SQL, device_id, enabled)
    }

    fn clipboard_sync(&self, device_id: &str) -> bool {
        self.read_flag(SELECT_CLIPBOARD_SYNC_SQL, device_id)
    }

    fn set_clipboard_sync(&self, device_id: &str, enabled: bool) -> Result<()> {
        self.write_flag(UPSERT_CLIPBOARD_SYNC_SQL, device_id, enabled)
    }
}

impl SqliteTrustStore {
    // Settings only count while the device is still trusted (the SELECTs join on trusted_devices).
    fn read_flag(&self, sql: &str, device_id: &str) -> bool {
        let Ok(connection) = self.lock() else { return false };
        connection.query_row(sql, params![device_id], |row| row.get::<_, bool>(0)).unwrap_or(false)
    }

    fn write_flag(&self, sql: &str, device_id: &str, enabled: bool) -> Result<()> {
        self.lock()?.execute(sql, params![device_id, enabled]).map_err(query_error)?;
        Ok(())
    }
}

// Older databases lack newer columns; adding one that exists is not an error.
fn add_column_if_missing(connection: &Connection, sql: &str) -> Result<()> {
    match connection.execute(sql, []) {
        Ok(_) => Ok(()),
        Err(error) if error.to_string().contains(DUPLICATE_COLUMN_ERROR) => Ok(()),
        Err(error) => Err(db_error(msg::OPEN_FAILED, error)),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn device(id: &str, paired_at: u64) -> TrustedDevice {
        TrustedDevice { id: id.into(), name: format!("name-{id}"), os: "linux".into(), paired_at }
    }

    #[test]
    fn auto_accept_needs_trust_and_is_cleared_on_remove() {
        let dir = TempDir::new().unwrap();
        let store = SqliteTrustStore::open(&dir.path().join("trust.db")).unwrap();
        store.set_auto_accept("a", true).unwrap();
        assert!(!store.auto_accept("a"));
        store.add(device("a", 1)).unwrap();
        assert!(store.auto_accept("a"));
        store.remove("a").unwrap();
        store.add(device("a", 2)).unwrap();
        assert!(!store.auto_accept("a"));
    }

    #[test]
    fn clipboard_sync_is_independent_of_auto_accept_and_survives_reopen() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("trust.db");
        let store = SqliteTrustStore::open(&path).unwrap();
        store.add(device("a", 1)).unwrap();
        store.set_auto_accept("a", true).unwrap();
        store.set_clipboard_sync("a", true).unwrap();
        store.set_auto_accept("a", false).unwrap();
        drop(store);
        let reopened = SqliteTrustStore::open(&path).unwrap();
        assert!(reopened.clipboard_sync("a"));
        assert!(!reopened.auto_accept("a"));
    }

    #[test]
    fn add_list_remove_round_trip() {
        let dir = TempDir::new().unwrap();
        let store = SqliteTrustStore::open(&dir.path().join("trust.db")).unwrap();
        store.add(device("a", 2)).unwrap();
        store.add(device("b", 1)).unwrap();
        assert!(store.is_trusted("a"));
        assert_eq!(store.list().unwrap(), vec![device("b", 1), device("a", 2)]);
        assert!(store.remove("a").unwrap());
        assert!(!store.remove("a").unwrap());
        assert!(!store.is_trusted("a"));
    }

    #[test]
    fn add_replaces_existing_device() {
        let dir = TempDir::new().unwrap();
        let store = SqliteTrustStore::open(&dir.path().join("trust.db")).unwrap();
        store.add(device("a", 1)).unwrap();
        store.add(TrustedDevice { name: "renamed".into(), ..device("a", 5) }).unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "renamed");
    }

    #[test]
    fn data_survives_reopen() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("trust.db");
        SqliteTrustStore::open(&path).unwrap().add(device("a", 1)).unwrap();
        assert!(SqliteTrustStore::open(&path).unwrap().is_trusted("a"));
    }
}
