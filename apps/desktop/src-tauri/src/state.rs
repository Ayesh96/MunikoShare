//! Managed app state: the node plus the service that forwards its events.

use std::path::PathBuf;

use muniko_core::{Lifecycle, MunikoNode};
use tauri::AppHandle;
use tokio::sync::{Mutex, RwLock};

use crate::event_forwarder::EventForwarder;
use crate::hotkey::HotkeyService;
use crate::messages::logs;
use crate::node_config;

pub struct AppState {
    // Read lock for queries and sends, write lock only for start/stop.
    pub node: RwLock<MunikoNode>,
    /// Where this run saves received files; a changed setting only applies after a restart.
    pub download_dir: PathBuf,
    forwarder: Mutex<EventForwarder>,
    hotkey: Mutex<HotkeyService>,
}

impl AppState {
    pub async fn start(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let config = node_config::build(app)?;
        let download_dir = config.download_dir.clone();
        let mut node = MunikoNode::new(config).await?;
        let mut forwarder = EventForwarder::new(app.clone(), node.subscribe());
        node.start().await?;
        forwarder.start().await?;
        let mut hotkey = HotkeyService::new(app.clone());
        hotkey.start().await?;
        Ok(Self { node: RwLock::new(node), download_dir, forwarder: Mutex::new(forwarder), hotkey: Mutex::new(hotkey) })
    }

    /// Reverse of start: stop the hotkey, stop forwarding, then stop the node.
    pub async fn shutdown(&self) {
        if let Err(error) = self.hotkey.lock().await.stop().await {
            tracing::error!("{}: {}", logs::SHUTDOWN_FAILED, error);
        }
        if let Err(error) = self.forwarder.lock().await.stop().await {
            tracing::error!("{}: {}", logs::SHUTDOWN_FAILED, error);
        }
        if let Err(error) = self.node.write().await.stop().await {
            tracing::error!("{}: {}", logs::SHUTDOWN_FAILED, error);
        }
    }
}
