use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::util::lock::lock;
use crate::constants::clipboard::{RECENT_HASH_CAPACITY, RECENT_HASH_TTL};

/// Hashes of content written because a peer sent it, so the watcher does not echo it back.
#[derive(Clone, Default)]
pub struct RecentHashes {
    received: Arc<Mutex<VecDeque<(String, Instant)>>>,
}

impl RecentHashes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Call before writing peer content to the clipboard so the watcher cannot race it.
    pub fn remember_received(&self, hash: &str) {
        let mut received = lock(&self.received);
        prune(&mut received);
        received.push_back((hash.to_string(), Instant::now()));
        while received.len() > RECENT_HASH_CAPACITY {
            received.pop_front();
        }
    }

    /// True once per remembered hash; a later deliberate copy of the same content syncs again.
    pub fn take_received(&self, hash: &str) -> bool {
        let mut received = lock(&self.received);
        prune(&mut received);
        match received.iter().position(|(known, _)| known == hash) {
            Some(index) => received.remove(index).is_some(),
            None => false,
        }
    }
}

fn prune(received: &mut VecDeque<(String, Instant)>) {
    received.retain(|(_, at)| at.elapsed() < RECENT_HASH_TTL);
}
