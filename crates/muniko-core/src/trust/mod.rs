//! Which devices this device has paired with and therefore trusts.

mod memory;
mod sqlite;

use serde::{Deserialize, Serialize};

use crate::error::Result;

pub use memory::MemoryTrustStore;
pub use sqlite::SqliteTrustStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub id: String,
    pub name: String,
    pub os: String,
    /// Unix seconds.
    pub paired_at: u64,
}

/// Storage seam for trusted devices; features depend on this trait, not a concrete store.
pub trait TrustStore: Send + Sync {
    fn is_trusted(&self, device_id: &str) -> bool;
    fn list(&self) -> Result<Vec<TrustedDevice>>;
    fn add(&self, device: TrustedDevice) -> Result<()>;
    fn remove(&self, device_id: &str) -> Result<bool>;
    /// Whether offers from this device are accepted without asking.
    fn auto_accept(&self, device_id: &str) -> bool;
    fn set_auto_accept(&self, device_id: &str, enabled: bool) -> Result<()>;
    /// Whether clipboard changes are mirrored to this device automatically (off by default).
    fn clipboard_sync(&self, device_id: &str) -> bool;
    fn set_clipboard_sync(&self, device_id: &str, enabled: bool) -> Result<()>;
}
