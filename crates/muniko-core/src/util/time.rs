use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch, or 0 if the clock is before it.
pub fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
