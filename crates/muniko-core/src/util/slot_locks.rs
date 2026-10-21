use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use super::lock::lock;

/// One async lock per key, created on demand; lets work on different keys run in parallel.
#[derive(Default)]
pub struct SlotLocks {
    slots: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
}

impl SlotLocks {
    pub async fn lock(&self, key: &str) -> OwnedMutexGuard<()> {
        let slot = {
            let mut slots = lock(&self.slots);
            slots.retain(|_, slot| Arc::strong_count(slot) > 1);
            slots.entry(key.to_string()).or_default().clone()
        };
        slot.lock_owned().await
    }

    /// True while someone holds the key's lock.
    pub fn is_locked(&self, key: &str) -> bool {
        let slots = lock(&self.slots);
        slots.get(key).is_some_and(|slot| slot.try_lock().is_err())
    }
}
