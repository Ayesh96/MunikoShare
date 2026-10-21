use muniko_core::history::HistoryEntry;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn get_history(state: State<'_, AppState>, limit: usize) -> Result<Vec<HistoryEntry>, String> {
    state.node.read().await.history(limit).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state.node.read().await.clear_history().map_err(|error| error.to_string())
}
