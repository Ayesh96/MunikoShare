use std::time::Duration;

pub const ROUTE_BOARD: &str = "/api/v1/board";
pub const ROUTE_BOARD_JOIN: &str = "/api/v1/board/join";
pub const ROUTE_BOARD_DIGEST: &str = "/api/v1/board/digest";
pub const ROUTE_BOARD_FETCH: &str = "/api/v1/board/items/fetch";
pub const ROUTE_BOARD_ITEMS: &str = "/api/v1/board/items";
pub const ROUTE_BOARD_FILES: &str = "/api/v1/board/files";
pub const ROUTE_BOARD_FILE: &str = "/api/v1/board/files/{sha256}";

/// Items older than this are pruned (7 days).
pub const DEFAULT_RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// How long a device without a board waits for peers to show one before creating its own.
pub const DISCOVERY_GRACE: Duration = Duration::from_secs(4);
/// How often the bootstrap re-probes peers while waiting.
pub const BOOTSTRAP_PROBE_INTERVAL: Duration = Duration::from_millis(500);
pub const SYNC_INTERVAL: Duration = Duration::from_secs(10);
/// Upper bound for one sync exchange with one peer.
pub const PEER_SYNC_TIMEOUT: Duration = Duration::from_secs(20);

pub const MAX_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_FILE_NAME_CHARS: usize = 255;
pub const MAX_BOARD_NAME_CHARS: usize = 80;
pub const MAX_MEMBER_NAME_CHARS: usize = 80;
/// Most items accepted in one push request.
pub const MAX_PUSH_ITEMS: usize = 200;
/// Most ids requested in one fetch request.
pub const MAX_FETCH_IDS: usize = 100;
/// Most items one pull round collects, so a huge backlog is caught up over several rounds.
pub const MAX_PULL_ITEMS: usize = 1000;

/// Repeated hashing makes guessing a join code offline slower.
pub const JOIN_HASH_ROUNDS: u32 = 50_000;
pub const JOIN_SALT_BYTES: usize = 16;
pub const MIN_JOIN_CODE_CHARS: usize = 4;
pub const MAX_JOIN_FAILURES: u32 = 5;
pub const JOIN_FAILURE_WINDOW: Duration = Duration::from_secs(60);

pub const CANONICAL_DOMAIN: &str = "muniko-board-item-v1";
pub const KIND_TEXT: &str = "text";
pub const KIND_FILE: &str = "file";
pub const KIND_TOMBSTONE: &str = "tombstone";
pub const KIND_MEMBER: &str = "member";
pub const MODE_OPEN: &str = "open";
pub const MODE_PROTECTED: &str = "protected";

pub const CREATE_BOARDS_SQL: &str = "CREATE TABLE IF NOT EXISTS boards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    mode TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    code_salt TEXT,
    code_hash TEXT
)";
pub const CREATE_ITEMS_SQL: &str = "CREATE TABLE IF NOT EXISTS board_items (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    kind TEXT NOT NULL,
    hash TEXT NOT NULL,
    body TEXT NOT NULL
)";
pub const SELECT_BOARD_SQL: &str = "SELECT id, name, mode, created_at, code_salt, code_hash FROM boards LIMIT 1";
pub const REPLACE_BOARD_SQL: &str = "INSERT OR REPLACE INTO boards (id, name, mode, created_at, code_salt, code_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
pub const CLEAR_BOARDS_SQL: &str = "DELETE FROM boards";
pub const CLEAR_ITEMS_SQL: &str = "DELETE FROM board_items";
pub const INSERT_ITEM_SQL: &str = "INSERT OR IGNORE INTO board_items (id, board_id, created_at, kind, hash, body) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
pub const SELECT_ITEM_SQL: &str = "SELECT body FROM board_items WHERE id = ?1";
pub const SELECT_ALL_ITEMS_SQL: &str = "SELECT body FROM board_items ORDER BY id";
pub const SELECT_KIND_SQL: &str = "SELECT body FROM board_items WHERE kind = ?1 ORDER BY id";
pub const SELECT_DIGEST_SQL: &str = "SELECT id, hash, created_at FROM board_items ORDER BY id";
pub const DELETE_ITEM_SQL: &str = "DELETE FROM board_items WHERE id = ?1";
pub const PRUNE_SQL: &str = "DELETE FROM board_items WHERE created_at < ?1 AND kind != ?2";

pub const CREATE_SETTINGS_SQL: &str = "CREATE TABLE IF NOT EXISTS board_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
)";
pub const SELECT_SETTING_SQL: &str = "SELECT value FROM board_settings WHERE key = ?1";
pub const REPLACE_SETTING_SQL: &str = "INSERT OR REPLACE INTO board_settings (key, value) VALUES (?1, ?2)";
pub const SETTING_KEEPER: &str = "keeper";
pub const SETTING_ON: &str = "1";
pub const SETTING_OFF: &str = "0";

/// Folder under the data dir that holds cached board files, one file per SHA-256.
pub const FILES_DIR_NAME: &str = "board-files";
pub const PART_SUFFIX: &str = ".part";
pub const INGEST_PREFIX: &str = ".ingest-";
/// Files up to this size are cached by every member without being asked.
pub const AUTO_CACHE_LIMIT: u64 = 10 * 1024 * 1024;
/// Per-device cap on cached board files.
pub const DEFAULT_CACHE_CAP: u64 = 2 * 1024 * 1024 * 1024;
/// Cap for a keeper device, which caches everything that fits.
pub const KEEPER_CACHE_CAP: u64 = 64 * 1024 * 1024 * 1024;
pub const RANGE_UNIT_PREFIX: &str = "bytes=";
pub const ACCEPT_RANGES_VALUE: &str = "bytes";

/// MAX_PUSH_ITEMS items of up to MAX_TEXT_BYTES each, plus signing overhead.
pub const MAX_PUSH_BODY_BYTES: usize = 16 * 1024 * 1024;
