use std::time::Duration;


pub const SERVICE_TYPE: &str = "_muniko._tcp.local.";
pub const TXT_KEY_ID: &str = "id";
pub const TXT_KEY_NAME: &str = "name";
pub const TXT_KEY_OS: &str = "os";
pub const TXT_KEY_PROTOCOL: &str = "proto";
/// DNS labels are limited to 63 bytes and the device id is 64 hex chars.
pub const INSTANCE_NAME_LEN: usize = 32;
pub const HOST_NAME_SUFFIX: &str = ".local.";
/// Upper bound for waiting on the daemon to confirm shutdown.
pub const SHUTDOWN_TIMEOUT_MS: u64 = 2000;
#[cfg(test)]
pub const TEST_PEER_WAIT_MS: u64 = 8000;
#[cfg(test)]
pub const TEST_POLL_INTERVAL_MS: u64 = 100;
/// UDP fallback for networks that block multicast.
pub const BROADCAST_PORT: u16 = 53421;
pub const BROADCAST_INTERVAL: Duration = Duration::from_secs(3);
/// A peer seen only by broadcast is dropped after this much silence.
pub const PEER_TTL: Duration = Duration::from_secs(15);
pub const ANNOUNCE_TAG: &str = "muniko-share";
pub const BROADCAST_BUFFER_SIZE: usize = 2048;
#[cfg(test)]
pub const TEST_BROADCAST_PORT: u16 = 53931;
#[cfg(test)]
pub const TEST_BROADCAST_INTERVAL: Duration = Duration::from_millis(100);
#[cfg(test)]
pub const TEST_PEER_TTL: Duration = Duration::from_millis(400);
