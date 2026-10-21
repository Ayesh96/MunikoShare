use muniko_core::clipboard::HistorySummary;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn push_clipboard(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.node.read().await.push_clipboard(&peer_id).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn push_text(state: State<'_, AppState>, peer_id: String, text: String) -> Result<(), String> {
    state.node.read().await.push_text(&peer_id, &text).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_clipboard_sync(state: State<'_, AppState>, device_id: String, enabled: bool) -> Result<(), String> {
    state.node.read().await.set_clipboard_sync(&device_id, enabled).map_err(|error| error.to_string())
}

/// Ids of trusted devices with automatic clipboard sync switched on.
#[tauri::command]
pub async fn list_clipboard_sync(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let node = state.node.read().await;
    Ok(node.trusted_devices().into_iter().filter(|device| node.clipboard_sync(&device.id)).map(|device| device.id).collect())
}

#[tauri::command]
pub async fn clipboard_history(state: State<'_, AppState>) -> Result<Vec<HistorySummary>, String> {
    Ok(state.node.read().await.clipboard_history())
}

#[tauri::command]
pub async fn copy_clipboard_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.node.read().await.copy_clipboard_item(&id).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn clipboard_image(state: State<'_, AppState>, id: String) -> Result<Option<String>, String> {
    Ok(state.node.read().await.clipboard_image(&id))
}

#[tauri::command]
pub async fn pin_clipboard_item(state: State<'_, AppState>, id: String, pinned: bool) -> Result<bool, String> {
    Ok(state.node.read().await.pin_clipboard_item(&id, pinned))
}

#[tauri::command]
pub async fn clear_clipboard_history(state: State<'_, AppState>) -> Result<(), String> {
    state.node.read().await.clear_clipboard_history();
    Ok(())
}
