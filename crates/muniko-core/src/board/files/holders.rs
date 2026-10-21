//! Which peers last said they hold which files.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::device::DeviceInfo;
use crate::util::lock::lock;

#[derive(Default)]
pub struct Holders {
    by_peer: Mutex<HashMap<String, HashSet<String>>>,
}

impl Holders {
    /// Replaces what `peer_id` holds; true when that changed.
    pub fn update(&self, peer_id: &str, have: &[String]) -> bool {
        let fresh: HashSet<String> = have.iter().cloned().collect();
        lock(&self.by_peer).insert(peer_id.to_string(), fresh.clone()) != Some(fresh)
    }

    /// Online peers holding `sha256`, in the order of `online`.
    pub fn of(&self, sha256: &str, online: &[DeviceInfo]) -> Vec<DeviceInfo> {
        let by_peer = lock(&self.by_peer);
        online.iter().filter(|peer| by_peer.get(&peer.id).is_some_and(|have| have.contains(sha256))).cloned().collect()
    }

    /// Every hash some online peer holds.
    pub fn online_hashes(&self, online: &[DeviceInfo]) -> HashSet<String> {
        let by_peer = lock(&self.by_peer);
        online.iter().filter_map(|peer| by_peer.get(&peer.id)).flatten().cloned().collect()
    }
}
