//! Global shortcut that pushes the clipboard to every paired device in range.

use async_trait::async_trait;
use muniko_core::{Lifecycle, LifecycleState, MunikoNode, Result};
use serde::Serialize;
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::constants::app::{HOTKEY_EVENT_NAME, PUSH_CLIPBOARD_SHORTCUT};
use crate::messages::{errors, logs};
use crate::state::AppState;

/// Outcome shown to the user: how many devices got the clipboard and what failed.
#[derive(Serialize, Clone)]
struct HotkeyResult {
    sent: usize,
    errors: Vec<String>,
}

pub struct HotkeyService {
    app: AppHandle,
    state: LifecycleState,
}

impl HotkeyService {
    pub fn new(app: AppHandle) -> Self {
        Self { app, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for HotkeyService {
    async fn start(&mut self) -> Result<()> {
        let handle = self.app.clone();
        let registered = self.app.global_shortcut().on_shortcut(PUSH_CLIPBOARD_SHORTCUT, move |_, _, event| {
            if event.state == ShortcutState::Pressed {
                spawn(push_and_report(handle.clone()));
            }
        });
        match registered {
            Ok(()) => {
                self.state = LifecycleState::Running;
                tracing::info!("{}", logs::HOTKEY_REGISTERED);
            }
            // The shortcut may be owned by another app; the rest of the app still works.
            Err(error) => tracing::warn!(%error, "{}", logs::HOTKEY_REGISTER_FAILED),
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if self.state == LifecycleState::Running {
            if let Err(error) = self.app.global_shortcut().unregister(PUSH_CLIPBOARD_SHORTCUT) {
                tracing::warn!(%error, "{}", logs::HOTKEY_UNREGISTER_FAILED);
            }
        }
        self.state = LifecycleState::Stopped;
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

async fn push_and_report(app: AppHandle) {
    let Some(state) = app.try_state::<AppState>() else { return };
    let result = push_to_nearby_paired(&*state.node.read().await).await;
    if let Err(error) = app.emit(HOTKEY_EVENT_NAME, result) {
        tracing::warn!(%error, "{}", errors::EMIT_FAILED);
    }
}

async fn push_to_nearby_paired(node: &MunikoNode) -> HotkeyResult {
    let trusted: Vec<String> = node.trusted_devices().into_iter().map(|device| device.id).collect();
    let mut result = HotkeyResult { sent: 0, errors: Vec::new() };
    for peer in node.peers().into_iter().filter(|peer| trusted.contains(&peer.id)) {
        match node.push_clipboard(&peer.id).await {
            Ok(()) => result.sent += 1,
            Err(error) => {
                tracing::warn!(%error, "{}", logs::HOTKEY_PUSH_FAILED);
                result.errors.push(format!("{}: {}", peer.name, error));
            }
        }
    }
    result
}
