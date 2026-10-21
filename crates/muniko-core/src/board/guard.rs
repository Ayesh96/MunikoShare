use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::constants::board::{JOIN_FAILURE_WINDOW, MAX_JOIN_FAILURES};

/// Limits wrong join codes per device so a short code cannot be brute-forced.
#[derive(Default)]
pub struct JoinGuard {
    failures: Mutex<HashMap<String, (Instant, u32)>>,
}

impl JoinGuard {
    pub fn allows(&self, device_id: &str) -> bool {
        let failures = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        match failures.get(device_id) {
            Some((since, count)) => since.elapsed() > JOIN_FAILURE_WINDOW || *count < MAX_JOIN_FAILURES,
            None => true,
        }
    }

    pub fn record_failure(&self, device_id: &str) {
        let mut failures = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        let entry = failures.entry(device_id.to_string()).or_insert((Instant::now(), 0));
        if entry.0.elapsed() > JOIN_FAILURE_WINDOW {
            *entry = (Instant::now(), 0);
        }
        entry.1 += 1;
    }

    pub fn clear(&self, device_id: &str) {
        self.failures.lock().unwrap_or_else(|e| e.into_inner()).remove(device_id);
    }
}
