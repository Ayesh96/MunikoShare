//! Paths handed to the app at launch (CLI, "Open with", "Send to") wait here until the UI drains them.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::constants::app::LAUNCH_EVENT_NAME;
use crate::messages::logs;
use crate::tray;

#[derive(Default)]
pub struct LaunchQueue {
    paths: Mutex<Vec<String>>,
}

impl LaunchQueue {
    pub fn push(&self, mut paths: Vec<String>) {
        if let Ok(mut queue) = self.paths.lock() {
            queue.append(&mut paths);
        }
    }

    pub fn take(&self) -> Vec<String> {
        self.paths.lock().map(|mut queue| std::mem::take(&mut *queue)).unwrap_or_default()
    }
}

/// Queues the paths, tells the UI, and brings the window forward.
pub fn deliver(app: &AppHandle, paths: Vec<String>) {
    if paths.is_empty() {
        return;
    }
    tracing::info!(count = paths.len(), "{}", logs::LAUNCH_PATHS_QUEUED);
    app.state::<LaunchQueue>().push(paths);
    if let Err(err) = app.emit(LAUNCH_EVENT_NAME, ()) {
        tracing::warn!(%err, "{}", crate::messages::errors::EMIT_FAILED);
    }
    tray::show_main_window(app);
}
