//! Builds the NodeConfig from the OS environment and the saved settings.

use std::path::PathBuf;

use muniko_core::NodeConfig;
use tauri::{AppHandle, Manager};

use crate::constants::app::DOWNLOAD_SUBDIR;
use crate::messages::{errors, logs};
use crate::settings_store::SettingsStore;

pub fn build(app: &AppHandle) -> Result<NodeConfig, String> {
    let data_dir = data_dir(app)?;
    let settings = SettingsStore::new(&data_dir).load();
    let device_name = settings.device_name.unwrap_or_else(default_device_name);
    let download_dir = match settings.download_dir {
        Some(dir) => dir,
        None => default_download_dir(app)?,
    };
    Ok(NodeConfig { device_name, data_dir, download_dir })
}

pub fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|_| errors::NO_DATA_DIR.to_string())
}

pub fn default_download_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let downloads = app.path().download_dir().map_err(|_| errors::NO_DOWNLOAD_DIR.to_string())?;
    Ok(downloads.join(DOWNLOAD_SUBDIR))
}

pub fn default_device_name() -> String {
    hostname::get()
        .ok()
        .and_then(|name| name.into_string().ok())
        .unwrap_or_else(|| logs::DEFAULT_DEVICE_NAME.to_string())
}
