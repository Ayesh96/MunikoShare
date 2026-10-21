use std::sync::{Arc, RwLock};

use crate::device::DeviceInfo;

/// This device's info, shared so the node can set the port once the server has bound it.
#[derive(Debug, Clone)]
pub struct LocalDevice {
    info: Arc<RwLock<DeviceInfo>>,
}

impl LocalDevice {
    pub fn new(info: DeviceInfo) -> Self {
        Self { info: Arc::new(RwLock::new(info)) }
    }

    pub fn set_port(&self, port: u16) {
        if let Ok(mut info) = self.info.write() {
            info.port = port;
        }
    }

    /// Current info; addresses are always empty because each peer sees us at a different one.
    pub fn snapshot(&self) -> Option<DeviceInfo> {
        let info = self.info.read().ok()?;
        Some(DeviceInfo { addresses: Vec::new(), ..info.clone() })
    }
}
