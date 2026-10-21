// Strings for the discovery module.
pub const STARTED: &str = "Discovery started";
pub const STOPPED: &str = "Discovery stopped";
pub const PEER_FOUND: &str = "Peer found";
pub const PEER_LOST: &str = "Peer lost";
pub const PEER_INVALID: &str = "Ignoring service with invalid TXT records";
pub const EVENT_CHANNEL_CLOSED: &str = "mDNS event channel closed";
pub const SHUTDOWN_UNCONFIRMED: &str = "mDNS daemon shutdown was not confirmed";
pub const BROADCAST_STARTED: &str = "Broadcast fallback started";
pub const BROADCAST_UNAVAILABLE: &str = "Broadcast fallback unavailable, relying on mDNS";
pub const BROADCAST_BIND_FAILED: &str = "Could not bind the broadcast socket";
pub const BROADCAST_ENCODE_FAILED: &str = "Could not encode the announcement";
pub const BROADCAST_SEND_FAILED: &str = "Broadcast send failed";
pub const BROADCAST_INVALID: &str = "Ignoring invalid broadcast datagram";
pub const BROADCAST_RECV_FAILED: &str = "Broadcast receive failed";
pub const MANUAL_PEER_REJECTED: &str = "Ignoring manual peer without addresses or with our own id";
