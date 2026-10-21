/// Frontend listens to this event for every NodeEvent.
pub const NODE_EVENT_NAME: &str = "muniko://node-event";
pub const DOWNLOAD_SUBDIR: &str = "Muniko Share";
pub const MAIN_WINDOW_LABEL: &str = "main";
pub const TRAY_ID: &str = "main-tray";
pub const TRAY_MENU_OPEN_ID: &str = "tray-open";
pub const TRAY_MENU_QUIT_ID: &str = "tray-quit";
/// Frontend listens to this event for the result of the global hotkey push.
pub const HOTKEY_EVENT_NAME: &str = "muniko://hotkey-result";
pub const PUSH_CLIPBOARD_SHORTCUT: &str = "CommandOrControl+Shift+V";
/// Frontend listens to this event and then drains the launch queue.
pub const LAUNCH_EVENT_NAME: &str = "muniko://launch-paths";
