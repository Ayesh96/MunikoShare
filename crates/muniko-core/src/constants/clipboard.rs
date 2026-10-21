use std::time::Duration;

pub const ROUTE_CLIPBOARD: &str = "/api/v1/clipboard";
/// Largest text a peer may push; keeps one request from flooding memory and the UI.
pub const MAX_CLIPBOARD_BYTES: usize = 1024 * 1024;
/// Largest PNG that is pushed or auto-synced; bigger images are skipped.
pub const MAX_IMAGE_PNG_BYTES: usize = 10 * 1024 * 1024;
/// Cap on decoded pixels so a tiny PNG cannot expand into gigabytes.
pub const MAX_IMAGE_DECODED_BYTES: usize = 256 * 1024 * 1024;
/// Request body cap for the route: a max-size PNG as base64 plus JSON framing.
pub const MAX_REQUEST_BODY_BYTES: usize = 16 * 1024 * 1024;
pub const JSON_CONTENT_TYPE: &str = "application/json";

/// How often the watcher polls the local clipboard.
pub const WATCH_INTERVAL: Duration = Duration::from_millis(500);
/// Upper bound for one auto-send to one peer.
pub const SEND_TIMEOUT: Duration = Duration::from_secs(30);

/// How many received hashes the loop guard remembers.
pub const RECENT_HASH_CAPACITY: usize = 16;
/// A received hash stops suppressing the watcher after this long.
pub const RECENT_HASH_TTL: Duration = Duration::from_secs(60);

pub const HISTORY_CAPACITY: usize = 30;
pub const HISTORY_PREVIEW_CHARS: usize = 120;
pub const PNG_DATA_URL_PREFIX: &str = "data:image/png;base64,";

pub const HASH_TEXT_TAG: &[u8] = b"text\0";
pub const HASH_IMAGE_TAG: &[u8] = b"image\0";

/// Pasteboard types password managers set on macOS (nspasteboard.org).
pub const MACOS_SENSITIVE_TYPES: [&str; 2] = ["org.nspasteboard.ConcealedType", "org.nspasteboard.TransientType"];
/// Registered Windows clipboard format that clipboard managers must not record.
pub const WINDOWS_EXCLUDE_FORMAT: &str = "ExcludeClipboardContentFromMonitorProcessing";
