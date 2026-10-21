use muniko_core::trust::TrustedDevice;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn request_pairing(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.node.read().await.request_pairing(&peer_id).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn confirm_pairing(state: State<'_, AppState>, peer_id: String, code: String) -> Result<(), String> {
    state.node.read().await.confirm_pairing(&peer_id, &code).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn unpair(state: State<'_, AppState>, device_id: String) -> Result<(), String> {
    state.node.read().await.unpair(&device_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_trusted(state: State<'_, AppState>) -> Result<Vec<TrustedDevice>, String> {
    Ok(state.node.read().await.trusted_devices())
}

#[tauri::command]
pub async fn set_auto_accept(state: State<'_, AppState>, device_id: String, enabled: bool) -> Result<(), String> {
    state.node.read().await.set_auto_accept(&device_id, enabled).map_err(|error| error.to_string())
}

/// Ids of trusted devices whose incoming files are accepted automatically.
#[tauri::command]
pub async fn list_auto_accept(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let node = state.node.read().await;
    Ok(node.trusted_devices().into_iter().filter(|device| node.auto_accept(&device.id)).map(|device| device.id).collect())
}
