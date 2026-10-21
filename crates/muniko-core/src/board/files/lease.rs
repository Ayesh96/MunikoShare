//! A pin on one cached file: while a lease is alive, cleanup and eviction leave the file alone.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures_util::{Stream, StreamExt};

use crate::util::lock::lock;

/// Counts live leases per hash; shared by the cache and every lease it hands out.
#[derive(Default)]
pub struct Leases {
    held: Mutex<HashMap<String, usize>>,
}

impl Leases {
    pub fn is_leased(&self, sha256: &str) -> bool {
        lock(&self.held).contains_key(sha256)
    }

    fn acquire(self: &Arc<Self>, sha256: &str) -> FileLease {
        *lock(&self.held).entry(sha256.to_string()).or_default() += 1;
        FileLease { leases: self.clone(), sha256: sha256.to_string() }
    }

    fn release(&self, sha256: &str) {
        let mut held = lock(&self.held);
        if let Some(count) = held.get_mut(sha256) {
            *count -= 1;
            if *count == 0 {
                held.remove(sha256);
            }
        }
    }
}

#[must_use = "the file is only protected while the lease is held"]
pub struct FileLease {
    leases: Arc<Leases>,
    sha256: String,
}

impl FileLease {
    pub(super) fn new(leases: &Arc<Leases>, sha256: &str) -> Self {
        leases.acquire(sha256)
    }

    /// Keeps the lease alive for as long as the stream exists.
    pub fn hold<S: Stream + Send + 'static>(self, stream: S) -> impl Stream<Item = S::Item> + Send + 'static {
        stream.map(move |item| {
            let _pinned = &self;
            item
        })
    }
}

impl Drop for FileLease {
    fn drop(&mut self) {
        self.leases.release(&self.sha256);
    }
}

#[cfg(test)]
mod tests {
    use futures_util::stream;

    use super::super::cache::FileCache;

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn cache_with_file() -> (tempfile::TempDir, FileCache) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(SHA), b"data").unwrap();
        let cache = FileCache::new(dir.path().to_path_buf());
        (dir, cache)
    }

    #[test]
    fn a_leased_file_is_busy_and_survives_removal() {
        let (_dir, cache) = cache_with_file();
        let lease = cache.lease(SHA);
        assert!(cache.is_busy(SHA));
        cache.remove(SHA);
        assert!(cache.has(SHA));
        drop(lease);
        assert!(!cache.is_busy(SHA));
        cache.remove(SHA);
        assert!(!cache.has(SHA));
    }

    #[test]
    fn the_file_stays_pinned_until_the_last_lease_drops() {
        let (_dir, cache) = cache_with_file();
        let (first, second) = (cache.lease(SHA), cache.lease(SHA));
        drop(first);
        assert!(cache.is_busy(SHA));
        drop(second);
        assert!(!cache.is_busy(SHA));
    }

    #[tokio::test]
    async fn a_held_stream_keeps_the_lease_until_it_is_dropped() {
        let (_dir, cache) = cache_with_file();
        let held = cache.lease(SHA).hold(stream::iter([1, 2]));
        cache.remove(SHA);
        assert!(cache.has(SHA));
        drop(held);
        cache.remove(SHA);
        assert!(!cache.has(SHA));
    }
}
