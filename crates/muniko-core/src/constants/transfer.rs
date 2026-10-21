use std::time::Duration;

pub const ROUTE_TRANSFERS: &str = "/api/v1/transfers";
pub const ROUTE_TRANSFER_FILE: &str = "/api/v1/transfers/{transfer_id}/files/{index}";

pub const MAX_FILES_PER_OFFER: usize = 1000;
/// Sanity cap on one offer's total size (16 TiB).
pub const MAX_OFFER_BYTES: u64 = 1 << 44;
pub const MAX_PATH_DEPTH: usize = 32;
pub const MAX_RELATIVE_PATH_CHARS: usize = 1024;
/// Relative paths use this separator on the wire on every OS.
pub const PATH_SEPARATOR: char = '/';
pub const SHA256_HEX_LEN: usize = 64;

/// How long the receiver's user has to answer an offer.
pub const OFFER_TIMEOUT: Duration = Duration::from_secs(60);
/// Sender waits a little longer than the receiver so the 408 arrives first.
pub const OFFER_REQUEST_TIMEOUT: Duration = Duration::from_secs(75);
/// An accepted transfer with no upload activity for this long is forgotten.
pub const TRANSFER_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
/// How many times the sender resumes one interrupted upload.
pub const RESUME_ATTEMPTS: u32 = 3;
/// Backoff before resume attempt n is this times n.
pub const RESUME_BACKOFF: Duration = Duration::from_millis(250);
pub const OFFSET_PARAM: &str = "offset";
pub const PROGRESS_INTERVAL: Duration = Duration::from_millis(200);

pub const TEMP_FILE_PREFIX: &str = ".muniko-";
pub const TEMP_FILE_SUFFIX: &str = ".part";
pub const MAX_FILE_NAME_CHARS: usize = 200;
pub const FALLBACK_CHAR: char = '_';
pub const INVALID_NAME_CHARS: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];
pub const RESERVED_WINDOWS_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
    "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];


/// An offer lists up to MAX_FILES_PER_OFFER paths of MAX_RELATIVE_PATH_CHARS plus checksums.
pub const MAX_OFFER_BODY_BYTES: usize = 2 * 1024 * 1024;
