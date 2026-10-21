// Constants for the mobile module.
use std::time::Duration;

pub const ROUTE_MOBILE: &str = "/m";
pub const ROUTE_MOBILE_DIR: &str = "/m/";
pub const ROUTE_MOBILE_ASSET: &str = "/m/{*path}";
pub const ROUTE_SESSION: &str = "/api/m/session";
pub const ROUTE_UPLOAD: &str = "/api/m/upload";
pub const ROUTE_INBOX: &str = "/api/m/inbox";
pub const ROUTE_INBOX_ITEM: &str = "/api/m/inbox/{id}";
pub const ROUTE_TEXT: &str = "/api/m/text";
pub const ROUTE_BOARD: &str = "/api/m/board";
pub const ROUTE_BOARD_TEXT: &str = "/api/m/board/text";
pub const ROUTE_BOARD_FILES: &str = "/api/m/board/files";
pub const ROUTE_BOARD_FILE: &str = "/api/m/board/files/{id}";
pub const ROUTE_EVENTS: &str = "/api/m/events";
/// Fragment key the desktop puts the token under in the QR URL.
pub const INVITE_FRAGMENT: &str = "#t=";

pub const SESSION_COOKIE: &str = "muniko_session";
pub const COOKIE_ATTRIBUTES: &str = "HttpOnly; Secure; SameSite=Strict; Path=/";
pub const FILE_NAME_HEADER: &str = "x-muniko-file-name";

pub const TOKEN_BYTES: usize = 32;
pub const TOKEN_TTL: Duration = Duration::from_secs(5 * 60);
pub const SESSION_TTL: Duration = Duration::from_secs(12 * 60 * 60);
pub const INBOX_TTL: Duration = Duration::from_secs(24 * 60 * 60);
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(30);
pub const SSE_KEEP_ALIVE: Duration = Duration::from_secs(15);

/// Failed token redemptions allowed per IP within the window.
pub const MAX_FAILED_REDEEMS: usize = 10;
pub const REDEEM_WINDOW: Duration = Duration::from_secs(60);

pub const MAX_UPLOAD_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
/// JSON escaping can roughly double a text, so the body cap sits above the text cap.
pub const MAX_TEXT_BODY_BYTES: usize = 2 * MAX_TEXT_BYTES + 1024;
pub const MAX_SESSION_EVENTS: usize = 32;

pub const INDEX_FILE: &str = "index.html";
pub const SPA_PREFIX: &str = "/m/";
pub const CACHE_NO_CACHE: &str = "no-cache";
pub const CACHE_ASSETS: &str = "public, max-age=86400";
pub const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";
pub const BYTES_UNIT: &str = "bytes";
pub const RANGE_PREFIX: &str = "bytes=";

pub const EVENT_INBOX: &str = "inbox";
pub const EVENT_TEXT: &str = "text";
pub const EVENT_BOARD: &str = "board";
pub const EVENT_REVOKED: &str = "revoked";
pub const EVENT_DATA: &str = "{}";

/// Board changes closer together than this reach the phone as one event.
pub const BOARD_EVENT_THROTTLE: Duration = Duration::from_millis(300);
pub const UPLOAD_TEMP_PREFIX: &str = "muniko-board-upload-";

pub const PHONE_ID_PREFIX: &str = "phone:";
pub const UNKNOWN_DEVICE_LABEL: &str = "Phone";
pub const LABEL_SEPARATOR: &str = " - ";

pub const PAGE_NOT_BUILT: &str = "<!doctype html><meta charset=utf-8><meta name=viewport content=\"width=device-width,initial-scale=1\"><title>Muniko Share</title><body style=\"font-family:sans-serif;padding:2rem\"><h1>Muniko Share</h1><p>The mobile page is not built in this version of the desktop app.</p></body>";
