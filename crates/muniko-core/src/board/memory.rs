use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use super::model::{BoardItem, BoardRecord, DigestEntry};
use super::store::BoardStore;
use crate::constants::board::KIND_MEMBER;
use crate::board::BoardError;
use crate::error::{MunikoError, Result};
use crate::messages::board as msg;

#[derive(Default)]
struct State {
    board: Option<BoardRecord>,
    items: BTreeMap<String, BoardItem>,
    keeper: bool,
}

/// Volatile store for tests and for devices that do not persist the board.
#[derive(Default)]
pub struct MemoryBoardStore {
    state: Mutex<State>,
}

impl MemoryBoardStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> Result<MutexGuard<'_, State>> {
        self.state.lock().map_err(|_| MunikoError::from(BoardError::Storage(msg::DB_LOCK_POISONED.to_string())))
    }
}

impl BoardStore for MemoryBoardStore {
    fn load_board(&self) -> Result<Option<BoardRecord>> {
        Ok(self.lock()?.board.clone())
    }

    fn save_board(&self, record: &BoardRecord) -> Result<()> {
        self.lock()?.board = Some(record.clone());
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        let mut state = self.lock()?;
        *state = State { keeper: state.keeper, ..State::default() };
        Ok(())
    }

    fn insert(&self, item: &BoardItem) -> Result<bool> {
        let mut state = self.lock()?;
        if state.items.contains_key(&item.id) {
            return Ok(false);
        }
        state.items.insert(item.id.clone(), item.clone());
        Ok(true)
    }

    fn get(&self, id: &str) -> Result<Option<BoardItem>> {
        Ok(self.lock()?.items.get(id).cloned())
    }

    fn all(&self) -> Result<Vec<BoardItem>> {
        Ok(self.lock()?.items.values().cloned().collect())
    }

    fn by_kind(&self, kind: &str) -> Result<Vec<BoardItem>> {
        Ok(self.lock()?.items.values().filter(|item| item.kind.tag() == kind).cloned().collect())
    }

    fn digest(&self) -> Result<Vec<DigestEntry>> {
        let state = self.lock()?;
        Ok(state.items.values().map(|item| DigestEntry { id: item.id.clone(), hash: item.hash(), created_at: item.created_at }).collect())
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.lock()?.items.remove(id);
        Ok(())
    }

    fn keeper(&self) -> Result<bool> {
        Ok(self.lock()?.keeper)
    }

    fn set_keeper(&self, keeper: bool) -> Result<()> {
        self.lock()?.keeper = keeper;
        Ok(())
    }

    fn prune_before(&self, cutoff: u64) -> Result<usize> {
        let mut state = self.lock()?;
        let before = state.items.len();
        state.items.retain(|_, item| item.created_at >= cutoff || item.kind.tag() == KIND_MEMBER);
        Ok(before - state.items.len())
    }
}
