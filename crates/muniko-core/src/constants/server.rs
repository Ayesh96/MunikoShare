use std::time::Duration;

pub const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);
pub const BIND_HOST: &str = "0.0.0.0";

/// Cap for small JSON control bodies (pairing, join, redeem).
pub const MAX_SMALL_JSON_BYTES: usize = 16 * 1024;
