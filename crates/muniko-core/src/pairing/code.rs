use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use subtle::ConstantTimeEq;

use super::profile::PairingProfile;
use crate::constants::pairing::CODE_LENGTH;

/// Random zero-padded numeric code from the thread-local CSPRNG.
pub fn generate_code() -> String {
    let value = rand::random_range(0..10u32.pow(CODE_LENGTH));
    format!("{value:0width$}", width = CODE_LENGTH as usize)
}

struct Pending {
    code: String,
    expires_at: Instant,
    attempts: u32,
    profile: PairingProfile,
}

/// Result of checking a submitted code against the pending request.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted(PairingProfile),
    WrongCode,
    /// Expired or out of attempts; the pending entry is gone.
    Expired,
    NoPending,
}

/// Pending pairing requests keyed by requesting device id.
pub struct PendingBook {
    pending: Mutex<HashMap<String, Pending>>,
    ttl: Duration,
    max_attempts: u32,
}

impl PendingBook {
    pub fn new(ttl: Duration, max_attempts: u32) -> Self {
        Self { pending: Mutex::new(HashMap::new()), ttl, max_attempts }
    }

    /// Starts (or restarts) a pairing for the device and returns its code.
    pub fn begin(&self, device_id: &str, profile: PairingProfile) -> String {
        let code = generate_code();
        let entry = Pending { code: code.clone(), expires_at: Instant::now() + self.ttl, attempts: 0, profile };
        self.entries().insert(device_id.to_string(), entry);
        code
    }

    pub fn verify(&self, device_id: &str, code: &str) -> Verdict {
        let mut entries = self.entries();
        let Some(entry) = entries.get_mut(device_id) else { return Verdict::NoPending };
        if Instant::now() >= entry.expires_at {
            entries.remove(device_id);
            return Verdict::Expired;
        }
        if bool::from(entry.code.as_bytes().ct_eq(code.as_bytes())) {
            return entries.remove(device_id).map(|e| Verdict::Accepted(e.profile)).unwrap_or(Verdict::NoPending);
        }
        entry.attempts += 1;
        if entry.attempts >= self.max_attempts {
            entries.remove(device_id);
            return Verdict::Expired;
        }
        Verdict::WrongCode
    }

    fn entries(&self) -> std::sync::MutexGuard<'_, HashMap<String, Pending>> {
        self.pending.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
