//! Small JSON file with the user's saved preferences; applied on the next start.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::constants::settings::SETTINGS_FILE_NAME;
use crate::messages::{errors, logs};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub device_name: Option<String>,
    pub download_dir: Option<PathBuf>,
}

pub struct SettingsStore {
    file: PathBuf,
}

impl SettingsStore {
    pub fn new(data_dir: &Path) -> Self {
        Self { file: data_dir.join(SETTINGS_FILE_NAME) }
    }

    /// A missing or corrupt file yields defaults so startup never fails on it.
    pub fn load(&self) -> Settings {
        match std::fs::read(&self.file) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|err| {
                tracing::warn!(%err, "{}", logs::SETTINGS_LOAD_FAILED);
                Settings::default()
            }),
            Err(_) => Settings::default(),
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), String> {
        let json = serde_json::to_vec_pretty(settings).map_err(|err| format!("{}: {err}", errors::SETTINGS_SAVE_FAILED))?;
        if let Some(parent) = self.file.parent() {
            std::fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", errors::SETTINGS_SAVE_FAILED))?;
        }
        std::fs::write(&self.file, json).map_err(|err| format!("{}: {err}", errors::SETTINGS_SAVE_FAILED))
    }
}
