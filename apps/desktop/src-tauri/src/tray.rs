//! System tray: keeps the app running in the background while the window is hidden.

use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Window, WindowEvent};

use crate::constants::app::{MAIN_WINDOW_LABEL, TRAY_ID, TRAY_MENU_OPEN_ID, TRAY_MENU_QUIT_ID};
use crate::messages::tray as msg;

/// Builds the tray icon and its menu.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, TRAY_MENU_OPEN_ID, msg::TRAY_MENU_OPEN_LABEL, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, TRAY_MENU_QUIT_ID, msg::TRAY_MENU_QUIT_LABEL, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let Some(icon) = app.default_window_icon().cloned() else {
        tracing::warn!("{}", msg::TRAY_NO_ICON);
        return Ok(());
    };
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(msg::TRAY_TOOLTIP)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Hides the main window instead of closing it, so the node keeps running.
pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        if let Err(err) = window.hide() {
            tracing::warn!(%err, "{}", msg::TRAY_HIDE_FAILED);
        }
    }
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        TRAY_MENU_OPEN_ID => show_main_window(app),
        // exit() triggers RunEvent::Exit, which runs the node shutdown.
        TRAY_MENU_QUIT_ID => app.exit(0),
        _ => {}
    }
}

pub fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else { return };
    let result = window.show().and_then(|_| window.unminimize()).and_then(|_| window.set_focus());
    if let Err(err) = result {
        tracing::warn!(%err, "{}", msg::TRAY_SHOW_FAILED);
    }
}
