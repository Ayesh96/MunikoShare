//! Muniko Share desktop shell: wires the core node into Tauri.

mod commands;
mod constants;
mod event_forwarder;
mod hotkey;
mod launch;
mod messages;
mod node_config;
mod os_integration;
mod settings_store;
mod state;
mod tray;
mod util;

use std::path::PathBuf;

use tauri::{Manager, RunEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::launch::LaunchQueue;
use crate::state::AppState;
use crate::util::launch_args;

pub fn run() {
    let app = tauri::Builder::default()
        // Must be first: a second launch forwards its arguments here and exits.
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            launch::deliver(app, launch_args::paths_from_args(&args, &PathBuf::from(cwd)));
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(LaunchQueue::default())
        .setup(|app| {
            os_integration::register();
            let cwd = std::env::current_dir().unwrap_or_default();
            let args: Vec<String> = std::env::args().collect();
            launch::deliver(app.handle(), launch_args::paths_from_args(&args, &cwd));
            let state = tauri::async_runtime::block_on(AppState::start(app.handle()))?;
            app.manage(state);
            tray::install(app.handle())?;
            Ok(())
        })
        .on_window_event(tray::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::launch::take_launch_paths,
            commands::settings::get_settings,
            commands::settings::set_device_name,
            commands::settings::set_download_dir,
            commands::settings::open_download_dir,
            commands::device::get_local_device,
            commands::device::list_peers,
            commands::device::local_addresses,
            commands::device::add_peer_by_address,
            commands::history::get_history,
            commands::history::clear_history,
            commands::pairing::set_auto_accept,
            commands::pairing::list_auto_accept,
            commands::transfer::send_files,
            commands::transfer::respond_to_offer,
            commands::pairing::request_pairing,
            commands::pairing::confirm_pairing,
            commands::pairing::unpair,
            commands::pairing::list_trusted,
            commands::clipboard::push_clipboard,
            commands::clipboard::push_text,
            commands::clipboard::set_clipboard_sync,
            commands::clipboard::list_clipboard_sync,
            commands::clipboard::clipboard_history,
            commands::clipboard::copy_clipboard_item,
            commands::clipboard::clipboard_image,
            commands::clipboard::pin_clipboard_item,
            commands::clipboard::clear_clipboard_history,
            commands::board::board_state,
            commands::board::board_post_text,
            commands::board::board_post_files,
            commands::board::board_post_image,
            commands::board::board_copy_image,
            commands::board::board_image,
            commands::board::board_download,
            commands::board::board_delete,
            commands::board::board_create,
            commands::board::board_join,
            commands::board::board_leave,
            commands::board::board_set_keeper,
            commands::phone::phone_invite,
            commands::phone::phone_sessions,
            commands::phone::phone_revoke,
            commands::phone::phone_send_files,
            commands::phone::phone_share_text,
            commands::phone::phone_set_access,
            commands::phone::phone_access
        ])
        .build(tauri::generate_context!())
        .expect("failed to build the Tauri app");

    app.run(|handle, event| match event {
        RunEvent::Exit => {
            if let Some(state) = handle.try_state::<AppState>() {
                tauri::async_runtime::block_on(state.shutdown());
            }
        }
        // macOS delivers "Open With" files through this event, also while running.
        #[cfg(target_os = "macos")]
        RunEvent::Opened { urls } => launch::deliver(handle, launch_args::paths_from_urls(&urls)),
        _ => {}
    });
}
