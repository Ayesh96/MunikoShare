use std::net::IpAddr;

use serde::{Deserialize, Serialize};

/// A Muniko device: the local one or a peer found on the network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// SHA-256 fingerprint of the device certificate, hex encoded.
    pub id: String,
    pub name: String,
    pub os: String,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    pub protocol_version: u16,
}
