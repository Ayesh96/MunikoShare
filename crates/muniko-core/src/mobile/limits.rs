//! Tunable lifetimes and caps; defaults come from the constants.

use crate::constants::mobile::{INBOX_TTL, MAX_FAILED_REDEEMS, MAX_UPLOAD_BYTES, REDEEM_WINDOW, SESSION_TTL, TOKEN_TTL};

#[derive(Debug, Clone)]
pub struct MobileLimits {
    pub token_ttl_secs: u64,
    pub session_ttl_secs: u64,
    pub inbox_ttl_secs: u64,
    pub max_upload_bytes: u64,
    pub max_failed_redeems: usize,
    pub redeem_window_secs: u64,
}

impl Default for MobileLimits {
    fn default() -> Self {
        Self {
            token_ttl_secs: TOKEN_TTL.as_secs(),
            session_ttl_secs: SESSION_TTL.as_secs(),
            inbox_ttl_secs: INBOX_TTL.as_secs(),
            max_upload_bytes: MAX_UPLOAD_BYTES,
            max_failed_redeems: MAX_FAILED_REDEEMS,
            redeem_window_secs: REDEEM_WINDOW.as_secs(),
        }
    }
}
