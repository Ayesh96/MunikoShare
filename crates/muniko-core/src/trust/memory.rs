use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use super::{TrustStore, TrustedDevice};
use crate::error::Result;

/// In-memory store for tests and as a fallback.
#[derive(Default)]
pub struct MemoryTrustStore {
    devices: RwLock<HashMap<String, TrustedDevice>>,
    auto_accept: RwLock<HashSet<String>>,
    clipboard_sync: RwLock<HashSet<String>>,
}

impl MemoryTrustStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TrustStore for MemoryTrustStore {
    fn is_trusted(&self, device_id: &str) -> bool {
        self.devices.read().map(|devices| devices.contains_key(device_id)).unwrap_or(false)
    }

    fn list(&self) -> Result<Vec<TrustedDevice>> {
        Ok(self.devices.read().map(|devices| devices.values().cloned().collect()).unwrap_or_default())
    }

    fn add(&self, device: TrustedDevice) -> Result<()> {
        if let Ok(mut devices) = self.devices.write() {
            devices.insert(device.id.clone(), device);
        }
        Ok(())
    }

    fn remove(&self, device_id: &str) -> Result<bool> {
        for set in [&self.auto_accept, &self.clipboard_sync] {
            if let Ok(mut set) = set.write() {
                set.remove(device_id);
            }
        }
        Ok(self.devices.write().map(|mut devices| devices.remove(device_id).is_some()).unwrap_or(false))
    }

    fn auto_accept(&self, device_id: &str) -> bool {
        self.is_trusted(device_id) && contains(&self.auto_accept, device_id)
    }

    fn set_auto_accept(&self, device_id: &str, enabled: bool) -> Result<()> {
        toggle(&self.auto_accept, device_id, enabled);
        Ok(())
    }

    fn clipboard_sync(&self, device_id: &str) -> bool {
        self.is_trusted(device_id) && contains(&self.clipboard_sync, device_id)
    }

    fn set_clipboard_sync(&self, device_id: &str, enabled: bool) -> Result<()> {
        toggle(&self.clipboard_sync, device_id, enabled);
        Ok(())
    }
}

fn contains(set: &RwLock<HashSet<String>>, device_id: &str) -> bool {
    set.read().map(|set| set.contains(device_id)).unwrap_or(false)
}

fn toggle(set: &RwLock<HashSet<String>>, device_id: &str, enabled: bool) {
    if let Ok(mut set) = set.write() {
        if enabled {
            set.insert(device_id.to_string());
        } else {
            set.remove(device_id);
        }
    }
}
