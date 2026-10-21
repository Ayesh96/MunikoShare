use std::path::PathBuf;
use std::time::Duration;

use crate::constants::board::{AUTO_CACHE_LIMIT, DEFAULT_CACHE_CAP, DEFAULT_RETENTION, DISCOVERY_GRACE, FILES_DIR_NAME, KEEPER_CACHE_CAP, SYNC_INTERVAL};
use crate::messages::board::DEFAULT_BOARD_NAME;

/// Tunables of the board; tests shrink the durations and sizes.
#[derive(Debug, Clone)]
pub struct BoardConfig {
    pub retention: Duration,
    pub discovery_grace: Duration,
    pub sync_interval: Duration,
    pub default_name: String,
    /// Content-addressed cache of board files (`<data_dir>/board-files`).
    pub files_dir: PathBuf,
    /// Where `download_file` exports to.
    pub download_dir: PathBuf,
    pub auto_cache_limit: u64,
    pub cache_cap: u64,
    pub keeper_cap: u64,
}

impl BoardConfig {
    /// Default tunables with the cache under `data_dir` and exports to `download_dir`.
    pub fn in_dirs(data_dir: &std::path::Path, download_dir: PathBuf) -> Self {
        Self { files_dir: data_dir.join(FILES_DIR_NAME), download_dir, ..Self::default() }
    }
}

impl Default for BoardConfig {
    fn default() -> Self {
        let base = std::env::temp_dir().join(FILES_DIR_NAME);
        Self {
            retention: DEFAULT_RETENTION,
            discovery_grace: DISCOVERY_GRACE,
            sync_interval: SYNC_INTERVAL,
            default_name: DEFAULT_BOARD_NAME.to_string(),
            files_dir: base.join("cache"),
            download_dir: base.join("downloads"),
            auto_cache_limit: AUTO_CACHE_LIMIT,
            cache_cap: DEFAULT_CACHE_CAP,
            keeper_cap: KEEPER_CACHE_CAP,
        }
    }
}
