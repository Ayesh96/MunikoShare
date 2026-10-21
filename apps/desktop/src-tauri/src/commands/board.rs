use std::path::PathBuf;

use muniko_core::board::{Board, BoardItemView, BoardMode, ObservedBoard};
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Everything the Board tab renders, fetched in one call.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardState {
    board: Option<Board>,
    items: Vec<BoardItemView>,
    conflicts: Vec<ObservedBoard>,
    discovered_boards: Vec<ObservedBoard>,
    keeper: bool,
}

fn text(error: impl ToString) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn board_state(state: State<'_, AppState>) -> Result<BoardState, String> {
    let board = state.node.read().await.board();
    Ok(BoardState {
        board: board.board().map_err(text)?,
        items: board.items().map_err(text)?,
        conflicts: board.conflicts(),
        discovered_boards: board.discovered_boards(),
        keeper: board.is_keeper().map_err(text)?,
    })
}

#[tauri::command]
pub async fn board_post_text(state: State<'_, AppState>, text: String) -> Result<(), String> {
    let board = state.node.read().await.board();
    board.post_text(&text).map(|_| ()).map_err(self::text)
}

#[tauri::command]
pub async fn board_post_files(state: State<'_, AppState>, paths: Vec<String>) -> Result<(), String> {
    let board = state.node.read().await.board();
    for path in paths {
        board.post_file(&PathBuf::from(path)).await.map_err(text)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn board_download(state: State<'_, AppState>, item_id: String) -> Result<String, String> {
    let board = state.node.read().await.board();
    let path = board.download_file(&item_id).await.map_err(text)?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn board_delete(state: State<'_, AppState>, item_id: String) -> Result<(), String> {
    state.node.read().await.board().delete_item(&item_id).map_err(text)
}

#[tauri::command]
pub async fn board_create(state: State<'_, AppState>, name: String, mode: BoardMode, join_code: Option<String>) -> Result<(), String> {
    let board = state.node.read().await.board();
    board.create_board(&name, mode, join_code.as_deref()).map(|_| ()).map_err(text)
}

#[tauri::command]
pub async fn board_join(state: State<'_, AppState>, peer_id: String, code: Option<String>) -> Result<(), String> {
    let board = state.node.read().await.board();
    board.join_board(&peer_id, code.as_deref()).await.map(|_| ()).map_err(text)
}

#[tauri::command]
pub async fn board_leave(state: State<'_, AppState>) -> Result<(), String> {
    state.node.read().await.board().leave_board().map_err(text)
}

#[tauri::command]
pub async fn board_set_keeper(state: State<'_, AppState>, keeper: bool) -> Result<(), String> {
    state.node.read().await.board().set_keeper(keeper).map_err(text)
}
