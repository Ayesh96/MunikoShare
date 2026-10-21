use std::net::IpAddr;

use muniko_core::DeviceInfo;
use tauri::State;

use crate::messages::errors;
use crate::state::AppState;
use crate::util::local_addresses::lan_ipv4_addresses;

#[tauri::command]
pub async fn get_local_device(state: State<'_, AppState>) -> Result<DeviceInfo, String> {
    Ok(state.node.read().await.local_device())
}

#[tauri::command]
pub async fn list_peers(state: State<'_, AppState>) -> Result<Vec<DeviceInfo>, String> {
    Ok(state.node.read().await.peers())
}

#[tauri::command]
pub fn local_addresses() -> Vec<String> {
    lan_ipv4_addresses()
}

#[tauri::command]
pub async fn add_peer_by_address(state: State<'_, AppState>, ip: String, port: u16) -> Result<DeviceInfo, String> {
    let ip: IpAddr = ip.trim().parse().map_err(|_| errors::INVALID_IP.to_string())?;
    state.node.read().await.add_peer_by_address(ip, port).await.map_err(|error| error.to_string())
}
