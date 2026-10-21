use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};

use super::model::{BoardItem, BoardMode, BoardRecord, DigestEntry, JoinSecret, Board};
use super::store::BoardStore;
use crate::constants::board::*;
use crate::error::{MunikoError, Result};
use crate::messages::board as msg;

fn db_error(context: &str, detail: impl std::fmt::Display) -> MunikoError {
    MunikoError::Io(std::io::Error::other(format!("{context}: {detail}")))
}

fn query_error(error: impl std::fmt::Display) -> MunikoError {
    db_error(msg::DB_QUERY_FAILED, error)
}

/// The board and its items in the `boards` and `board_items` tables of a SQLite file.
pub struct SqliteBoardStore {
    connection: Mutex<Connection>,
}

impl SqliteBoardStore {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path).map_err(|e| db_error(msg::DB_OPEN_FAILED, e))?;
        for sql in [CREATE_BOARDS_SQL, CREATE_ITEMS_SQL, CREATE_SETTINGS_SQL] {
            connection.execute(sql, []).map_err(|e| db_error(msg::DB_OPEN_FAILED, e))?;
        }
        Ok(Self { connection: Mutex::new(connection) })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| db_error(msg::DB_QUERY_FAILED, msg::DB_LOCK_POISONED))
    }

    fn query_items(&self, sql: &str, args: impl rusqlite::Params) -> Result<Vec<BoardItem>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(sql).map_err(query_error)?;
        let bodies = statement.query_map(args, |row| row.get::<_, String>(0)).map_err(query_error)?;
        let mut items = Vec::new();
        for body in bodies {
            let body = body.map_err(query_error)?;
            items.push(serde_json::from_str(&body).map_err(|e| db_error(msg::DB_BAD_ROW, e))?);
        }
        Ok(items)
    }
}

fn read_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<BoardRecord> {
    let mode: String = row.get(2)?;
    let mode = BoardMode::parse(&mode)
        .ok_or_else(|| rusqlite::Error::InvalidColumnType(2, msg::DB_BAD_ROW.into(), rusqlite::types::Type::Text))?;
    let salt: Option<String> = row.get(4)?;
    let hash: Option<String> = row.get(5)?;
    let join_secret = salt.zip(hash).map(|(salt, hash)| JoinSecret { salt, hash });
    let board = Board { id: row.get(0)?, name: row.get(1)?, mode, created_at: row.get::<_, i64>(3)? as u64 };
    Ok(BoardRecord { board, join_secret })
}

impl BoardStore for SqliteBoardStore {
    fn load_board(&self) -> Result<Option<BoardRecord>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(SELECT_BOARD_SQL).map_err(query_error)?;
        let mut rows = statement.query_map([], read_record).map_err(query_error)?;
        rows.next().transpose().map_err(query_error)
    }

    fn save_board(&self, record: &BoardRecord) -> Result<()> {
        let secret = record.join_secret.as_ref();
        let created_at = i64::try_from(record.board.created_at).unwrap_or(i64::MAX);
        self.lock()?
            .execute(
                REPLACE_BOARD_SQL,
                params![record.board.id, record.board.name, record.board.mode.as_str(), created_at, secret.map(|s| &s.salt), secret.map(|s| &s.hash)],
            )
            .map_err(query_error)?;
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(query_error)?;
        tx.execute(CLEAR_BOARDS_SQL, []).map_err(query_error)?;
        tx.execute(CLEAR_ITEMS_SQL, []).map_err(query_error)?;
        tx.commit().map_err(query_error)
    }

    fn insert(&self, item: &BoardItem) -> Result<bool> {
        let body = serde_json::to_string(item).map_err(query_error)?;
        let created_at = i64::try_from(item.created_at).unwrap_or(i64::MAX);
        let changed = self
            .lock()?
            .execute(INSERT_ITEM_SQL, params![item.id, item.board_id, created_at, item.kind.tag(), item.hash(), body])
            .map_err(query_error)?;
        Ok(changed > 0)
    }

    fn get(&self, id: &str) -> Result<Option<BoardItem>> {
        Ok(self.query_items(SELECT_ITEM_SQL, params![id])?.into_iter().next())
    }

    fn all(&self) -> Result<Vec<BoardItem>> {
        self.query_items(SELECT_ALL_ITEMS_SQL, [])
    }

    fn by_kind(&self, kind: &str) -> Result<Vec<BoardItem>> {
        self.query_items(SELECT_KIND_SQL, params![kind])
    }

    fn digest(&self) -> Result<Vec<DigestEntry>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(SELECT_DIGEST_SQL).map_err(query_error)?;
        let rows = statement
            .query_map([], |row| Ok(DigestEntry { id: row.get(0)?, hash: row.get(1)?, created_at: row.get::<_, i64>(2)? as u64 }))
            .map_err(query_error)?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(query_error)
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.lock()?.execute(DELETE_ITEM_SQL, params![id]).map_err(query_error)?;
        Ok(())
    }

    fn keeper(&self) -> Result<bool> {
        let connection = self.lock()?;
        let value = connection.query_row(SELECT_SETTING_SQL, params![SETTING_KEEPER], |row| row.get::<_, String>(0)).optional().map_err(query_error)?;
        Ok(value.as_deref() == Some(SETTING_ON))
    }

    fn set_keeper(&self, keeper: bool) -> Result<()> {
        let value = if keeper { SETTING_ON } else { SETTING_OFF };
        self.lock()?.execute(REPLACE_SETTING_SQL, params![SETTING_KEEPER, value]).map_err(query_error)?;
        Ok(())
    }

    fn prune_before(&self, cutoff: u64) -> Result<usize> {
        let cutoff = i64::try_from(cutoff).unwrap_or(i64::MAX);
        self.lock()?.execute(PRUNE_SQL, params![cutoff, KIND_MEMBER]).map_err(query_error)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::board::signing::ItemSigner;
    use crate::board::model::ItemKind;

    #[test]
    fn persists_board_and_items_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("muniko.db");
        let record = BoardRecord {
            board: Board { id: "b".into(), name: "n".into(), mode: BoardMode::Protected, created_at: 5 },
            join_secret: Some(JoinSecret::create("code")),
        };
        let item = ItemSigner::new(Arc::new(crate::util::test_support::test_identity()))
            .sign("b", ItemKind::Text { text: "hi".into() })
            .unwrap();
        {
            let store = SqliteBoardStore::open(&path).unwrap();
            store.save_board(&record).unwrap();
            assert!(store.insert(&item).unwrap());
            assert!(!store.insert(&item).unwrap());
        }
        let store = SqliteBoardStore::open(&path).unwrap();
        assert_eq!(store.load_board().unwrap(), Some(record));
        assert_eq!(store.get(&item.id).unwrap(), Some(item.clone()));
        assert_eq!(store.digest().unwrap()[0].hash, item.hash());
        assert!(!store.keeper().unwrap());
        store.set_keeper(true).unwrap();
        store.delete(&item.id).unwrap();
        assert!(store.all().unwrap().is_empty());
        assert!(SqliteBoardStore::open(&path).unwrap().keeper().unwrap());
    }
}
