pub const FORWARDER_STARTED: &str = "node event forwarder started";
pub const FORWARDER_STOPPED: &str = "node event forwarder stopped";
pub const EVENTS_LAGGED: &str = "node event forwarder lagged, events dropped";
pub const SHUTDOWN_FAILED: &str = "node shutdown failed";
pub const DEFAULT_DEVICE_NAME: &str = "Muniko device";
pub const HOTKEY_REGISTERED: &str = "clipboard hotkey registered";
pub const HOTKEY_REGISTER_FAILED: &str = "clipboard hotkey unavailable, continuing without it";
pub const HOTKEY_UNREGISTER_FAILED: &str = "clipboard hotkey unregister failed";
pub const HOTKEY_PUSH_FAILED: &str = "clipboard hotkey push failed";
pub const SETTINGS_LOAD_FAILED: &str = "settings file unreadable, using defaults";
pub const LAUNCH_PATHS_QUEUED: &str = "paths queued from launch arguments";
#[cfg(windows)]
pub const SEND_TO_LINK_FAILED: &str = "could not create the Windows Send to shortcut";
