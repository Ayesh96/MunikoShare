use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::messages::errors;
use crate::node_config;
use crate::settings_store::SettingsStore;
use crate::state::AppState;

/// What the Settings tab shows; changes apply on the next start.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    device_name: String,
    download_dir: String,
}

fn store(app: &AppHandle) -> Result<SettingsStore, String> {
    Ok(SettingsStore::new(&node_config::data_dir(app)?))
}

fn view(app: &AppHandle) -> Result<SettingsView, String> {
    let settings = store(app)?.load();
    let download_dir = match settings.download_dir {
        Some(dir) => dir,
        None => node_config::default_download_dir(app)?,
    };
    Ok(SettingsView {
        device_name: settings.device_name.unwrap_or_else(node_config::default_device_name),
        download_dir: download_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<SettingsView, String> {
    view(&app)
}

/// An empty name restores the computer's own name.
#[tauri::command]
pub fn set_device_name(app: AppHandle, name: String) -> Result<SettingsView, String> {
    let store = store(&app)?;
    let mut settings = store.load();
    let name = name.trim();
    settings.device_name = (!name.is_empty()).then(|| name.to_string());
    store.save(&settings)?;
    view(&app)
}

/// `None` restores the default download folder.
#[tauri::command]
pub fn set_download_dir(app: AppHandle, path: Option<String>) -> Result<SettingsView, String> {
    let store = store(&app)?;
    let mut settings = store.load();
    settings.download_dir = match path {
        Some(path) => {
            let dir = PathBuf::from(path);
            if !dir.is_dir() {
                return Err(errors::NOT_A_FOLDER.to_string());
            }
            Some(dir)
        }
        None => None,
    };
    store.save(&settings)?;
    view(&app)
}

/// Opens the folder the running node saves into.
#[tauri::command]
pub fn open_download_dir(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    std::fs::create_dir_all(&state.download_dir).map_err(|err| format!("{}: {err}", errors::OPEN_DOWNLOADS_FAILED))?;
    app.opener()
        .open_path(state.download_dir.to_string_lossy(), None::<&str>)
        .map_err(|err| format!("{}: {err}", errors::OPEN_DOWNLOADS_FAILED))
}
