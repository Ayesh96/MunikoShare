use std::path::PathBuf;

use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn send_files(state: State<'_, AppState>, peer_id: String, paths: Vec<String>) -> Result<(), String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    // A read lock is shared, so a long send never blocks other commands.
    let node = state.node.read().await;
    node.send_files(&peer_id, &paths).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn respond_to_offer(state: State<'_, AppState>, offer_id: String, accept: bool) -> Result<(), String> {
    state.node.read().await.respond_to_offer(&offer_id, accept).map_err(|error| error.to_string())
}
