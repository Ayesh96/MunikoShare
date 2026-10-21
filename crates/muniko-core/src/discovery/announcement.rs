//! The compact UDP broadcast message that announces a device.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::constants::app::PROTOCOL_VERSION;
use crate::constants::discovery::ANNOUNCE_TAG;
use crate::device::DeviceInfo;
use crate::net::candidate_addresses;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Announcement {
    tag: String,
    id: String,
    name: String,
    os: String,
    port: u16,
    protocol_version: u16,
}

impl Announcement {
    pub fn from_device(device: &DeviceInfo) -> Self {
        Self {
            tag: ANNOUNCE_TAG.to_string(),
            id: device.id.clone(),
            name: device.name.clone(),
            os: device.os.clone(),
            port: device.port,
            protocol_version: device.protocol_version,
        }
    }

    pub fn encode(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    /// None for anything that is not a Muniko announcement of our protocol version.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let announcement: Self = serde_json::from_slice(bytes).ok()?;
        (announcement.tag == ANNOUNCE_TAG && announcement.protocol_version == PROTOCOL_VERSION).then_some(announcement)
    }

    /// The sender's address comes from the datagram, not from the payload.
    pub fn into_device(self, sender: IpAddr) -> DeviceInfo {
        DeviceInfo {
            id: self.id,
            name: self.name,
            os: self.os,
            addresses: candidate_addresses(&[sender]),
            port: self.port,
            protocol_version: self.protocol_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> DeviceInfo {
        DeviceInfo { id: "abc".into(), name: "Laptop".into(), os: "macos".into(), addresses: vec![], port: 53420, protocol_version: PROTOCOL_VERSION }
    }

    #[test]
    fn round_trips_and_takes_the_address_from_the_sender() {
        let bytes = Announcement::from_device(&device()).encode().unwrap();
        let sender: IpAddr = "192.168.1.9".parse().unwrap();
        let decoded = Announcement::decode(&bytes).unwrap().into_device(sender);
        assert_eq!(decoded, DeviceInfo { addresses: vec![sender], ..device() });
    }

    #[test]
    fn rejects_foreign_tag_version_and_garbage() {
        let mut other_tag = Announcement::from_device(&device());
        other_tag.tag = "other".into();
        let mut other_version = Announcement::from_device(&device());
        other_version.protocol_version += 1;
        assert!(Announcement::decode(&other_tag.encode().unwrap()).is_none());
        assert!(Announcement::decode(&other_version.encode().unwrap()).is_none());
        assert!(Announcement::decode(b"not json").is_none());
    }
}
