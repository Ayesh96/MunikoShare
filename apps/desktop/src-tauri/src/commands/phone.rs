use std::net::IpAddr;
use std::path::PathBuf;

use muniko_core::mobile::{Invite, PhoneSession};
use tauri::State;

use crate::state::AppState;
use crate::util::local_addresses::lan_ipv4_addresses;

#[tauri::command]
pub async fn phone_invite(state: State<'_, AppState>) -> Result<Invite, String> {
    let addresses: Vec<IpAddr> = lan_ipv4_addresses().iter().filter_map(|text| text.parse().ok()).collect();
    state.node.read().await.create_phone_invite(&addresses).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn phone_sessions(state: State<'_, AppState>) -> Result<Vec<PhoneSession>, String> {
    Ok(state.node.read().await.phone_sessions())
}

#[tauri::command]
pub async fn phone_revoke(state: State<'_, AppState>, session_id: String) -> Result<bool, String> {
    Ok(state.node.read().await.revoke_phone(&session_id))
}

#[tauri::command]
pub async fn phone_send_files(state: State<'_, AppState>, session_id: String, paths: Vec<String>) -> Result<usize, String> {
    let paths = paths.into_iter().map(PathBuf::from).collect();
    state.node.read().await.send_files_to_phone(&session_id, paths).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn phone_share_text(state: State<'_, AppState>, session_id: String, text: String) -> Result<(), String> {
    state.node.read().await.share_text_to_phone(&session_id, text).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn phone_set_access(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    state.node.read().await.set_phone_access(enabled);
    Ok(())
}

#[tauri::command]
pub async fn phone_access(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.node.read().await.phone_access())
}
