/// Hard cap on stored rows; oldest are pruned first.
pub const MAX_HISTORY_ENTRIES: usize = 1000;
/// Characters of clipboard text kept in a history entry.
pub const CLIPBOARD_PREVIEW_CHARS: usize = 80;

pub const CREATE_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    kind TEXT NOT NULL,
    device_id TEXT NOT NULL,
    detail TEXT NOT NULL
)";
pub const INSERT_SQL: &str = "INSERT INTO history (timestamp, kind, device_id, detail) VALUES (?1, ?2, ?3, ?4)";
pub const SELECT_RECENT_SQL: &str = "SELECT id, timestamp, kind, device_id, detail FROM history ORDER BY id DESC LIMIT ?1";
pub const PRUNE_SQL: &str = "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY id DESC LIMIT ?1)";
pub const CLEAR_SQL: &str = "DELETE FROM history";
