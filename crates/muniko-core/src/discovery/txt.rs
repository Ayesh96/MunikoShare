//! Converts a DeviceInfo to and from mDNS TXT records.

use std::collections::HashMap;
use std::net::IpAddr;

use crate::constants::discovery::{TXT_KEY_ID, TXT_KEY_NAME, TXT_KEY_OS, TXT_KEY_PROTOCOL};
use crate::device::DeviceInfo;

pub fn encode(device: &DeviceInfo) -> HashMap<String, String> {
    HashMap::from([
        (TXT_KEY_ID.to_string(), device.id.clone()),
        (TXT_KEY_NAME.to_string(), device.name.clone()),
        (TXT_KEY_OS.to_string(), device.os.clone()),
        (TXT_KEY_PROTOCOL.to_string(), device.protocol_version.to_string()),
    ])
}

/// Returns None when a required record is missing or malformed.
pub fn decode(records: &HashMap<String, String>, addresses: Vec<IpAddr>, port: u16) -> Option<DeviceInfo> {
    Some(DeviceInfo {
        id: records.get(TXT_KEY_ID)?.clone(),
        name: records.get(TXT_KEY_NAME)?.clone(),
        os: records.get(TXT_KEY_OS)?.clone(),
        addresses,
        port,
        protocol_version: records.get(TXT_KEY_PROTOCOL)?.parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> DeviceInfo {
        DeviceInfo {
            id: "abc123".into(),
            name: "Laptop".into(),
            os: "macos".into(),
            addresses: vec!["192.168.1.5".parse().unwrap()],
            port: 53420,
            protocol_version: 1,
        }
    }

    #[test]
    fn round_trips() {
        let original = device();
        let decoded = decode(&encode(&original), original.addresses.clone(), original.port);
        assert_eq!(decoded, Some(original));
    }

    #[test]
    fn rejects_missing_or_bad_records() {
        let mut records = encode(&device());
        records.insert(TXT_KEY_PROTOCOL.to_string(), "x".into());
        assert!(decode(&records, vec![], 1).is_none());
        records.remove(TXT_KEY_ID);
        assert!(decode(&records, vec![], 1).is_none());
    }
}
