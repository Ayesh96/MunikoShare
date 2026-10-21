use super::model::{BoardItem, BoardRecord, DigestEntry};
use crate::error::Result;

/// Persistence for the one board this device is on and its item log.
pub trait BoardStore: Send + Sync {
    fn load_board(&self) -> Result<Option<BoardRecord>>;
    fn save_board(&self, record: &BoardRecord) -> Result<()>;
    /// Removes the board and every item.
    fn clear(&self) -> Result<()>;
    /// Returns false when an item with this id already exists.
    fn insert(&self, item: &BoardItem) -> Result<bool>;
    fn get(&self, id: &str) -> Result<Option<BoardItem>>;
    fn all(&self) -> Result<Vec<BoardItem>>;
    fn by_kind(&self, kind: &str) -> Result<Vec<BoardItem>>;
    fn digest(&self) -> Result<Vec<DigestEntry>>;
    fn delete(&self, id: &str) -> Result<()>;
    /// Whether this device caches every board file; survives leaving a board.
    fn keeper(&self) -> Result<bool>;
    fn set_keeper(&self, keeper: bool) -> Result<()>;
    /// Deletes everything created before `cutoff` except member records; returns the count.
    fn prune_before(&self, cutoff: u64) -> Result<usize>;
}
