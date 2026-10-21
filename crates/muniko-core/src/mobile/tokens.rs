//! One-time QR tokens, kept only as digests.

use std::collections::HashMap;

use super::secret::{digest, random_secret};
use crate::error::Result;

#[derive(Default)]
pub struct InviteTokens {
    expiry_by_digest: HashMap<String, u64>,
}

impl InviteTokens {
    /// Returns the new token and its expiry (unix seconds).
    pub fn issue(&mut self, now: u64, ttl_secs: u64) -> Result<(String, u64)> {
        let token = random_secret()?;
        let expires_at = now + ttl_secs;
        self.expiry_by_digest.insert(digest(&token), expires_at);
        Ok((token, expires_at))
    }

    /// Consumes the token; true only the first time and before it expires.
    pub fn redeem(&mut self, token: &str, now: u64) -> bool {
        self.expiry_by_digest.remove(&digest(token)).is_some_and(|expires_at| now < expires_at)
    }

    pub fn purge(&mut self, now: u64) {
        self.expiry_by_digest.retain(|_, expires_at| now < *expires_at);
    }

    pub fn clear(&mut self) {
        self.expiry_by_digest.clear();
    }
}
