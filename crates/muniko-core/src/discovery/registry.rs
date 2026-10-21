//! Peer map shared by every discovery source: merges addresses and tracks who vouches for each peer.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::device::DeviceInfo;
use crate::events::{EventBus, NodeEvent};
use crate::messages::discovery as messages;
use crate::net::candidate_addresses;

/// Who currently vouches for a peer; it is dropped once nobody does.
#[derive(Default)]
struct Sources {
    mdns_name: Option<String>,
    manual: bool,
    broadcast_seen: Option<Instant>,
}

impl Sources {
    fn any(&self) -> bool {
        self.mdns_name.is_some() || self.manual || self.broadcast_seen.is_some()
    }
}

struct Entry {
    device: DeviceInfo,
    sources: Sources,
}

pub struct PeerRegistry {
    local_id: String,
    events: EventBus,
    peers: Mutex<HashMap<String, Entry>>,
}

impl PeerRegistry {
    pub fn new(local_id: String, events: EventBus) -> Self {
        Self { local_id, events, peers: Mutex::default() }
    }

    pub fn snapshot(&self) -> Vec<DeviceInfo> {
        self.peers.lock().map(|peers| peers.values().map(|entry| entry.device.clone()).collect()).unwrap_or_default()
    }

    pub fn observe_mdns(&self, fullname: String, device: DeviceInfo) {
        let merged = self.observe(device, |sources| sources.mdns_name = Some(fullname));
        self.publish_found(merged);
    }

    pub fn observe_broadcast(&self, device: DeviceInfo, now: Instant) {
        let merged = self.observe(device, |sources| sources.broadcast_seen = Some(now));
        self.publish_found(merged);
    }

    /// Always publishes PeerAdded so the UI can confirm the add.
    pub fn add_manual(&self, device: DeviceInfo) {
        match self.observe(device, |sources| sources.manual = true) {
            Some((device, _)) => self.events.publish(NodeEvent::PeerAdded { device }),
            None => tracing::debug!("{}", messages::MANUAL_PEER_REJECTED),
        }
    }

    pub fn remove_manual(&self, device_id: &str) {
        self.release(|id, entry| id == device_id && entry.sources.manual, |sources| sources.manual = false);
    }

    pub fn mdns_removed(&self, fullname: &str) {
        self.release(|_, entry| entry.sources.mdns_name.as_deref() == Some(fullname), |sources| sources.mdns_name = None);
    }

    pub fn expire_broadcast(&self, ttl: Duration, now: Instant) {
        let stale = |_: &str, entry: &Entry| {
            entry.sources.broadcast_seen.is_some_and(|seen| now.saturating_duration_since(seen) > ttl)
        };
        self.release(stale, |sources| sources.broadcast_seen = None);
    }

    pub fn clear(&self) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.clear();
        }
    }

    /// Merges the sighting into the map; returns the merged device and whether anything changed.
    fn observe(&self, device: DeviceInfo, mark: impl FnOnce(&mut Sources)) -> Option<(DeviceInfo, bool)> {
        if device.id == self.local_id {
            return None;
        }
        let mut peers = self.peers.lock().ok()?;
        let known = peers.get(&device.id).map(|entry| &entry.device);
        let merged = merge(known, device);
        if merged.addresses.is_empty() {
            return None;
        }
        let changed = known != Some(&merged);
        let entry = peers.entry(merged.id.clone()).or_insert_with(|| Entry { device: merged.clone(), sources: Sources::default() });
        entry.device = merged.clone();
        mark(&mut entry.sources);
        Some((merged, changed))
    }

    fn publish_found(&self, merged: Option<(DeviceInfo, bool)>) {
        if let Some((device, true)) = merged {
            tracing::debug!("{}: {}", messages::PEER_FOUND, device.id);
            self.events.publish(NodeEvent::PeerFound { device });
        }
    }

    /// Drops one source from matching peers; peers nobody vouches for any more are lost.
    fn release(&self, matches: impl Fn(&str, &Entry) -> bool, clear: impl Fn(&mut Sources)) {
        let mut lost = Vec::new();
        if let Ok(mut peers) = self.peers.lock() {
            for (id, entry) in peers.iter_mut().filter(|(id, entry)| matches(id, entry)) {
                clear(&mut entry.sources);
                if !entry.sources.any() {
                    lost.push(id.clone());
                }
            }
            for id in &lost {
                peers.remove(id);
            }
        }
        for device_id in lost {
            tracing::debug!("{}: {device_id}", messages::PEER_LOST);
            self.events.publish(NodeEvent::PeerLost { device_id });
        }
    }
}

/// Sightings can carry partial address sets, so keep every usable address seen.
fn merge(known: Option<&DeviceInfo>, mut device: DeviceInfo) -> DeviceInfo {
    let earlier = known.map(|known| known.addresses.as_slice()).unwrap_or_default();
    let all: Vec<IpAddr> = earlier.iter().chain(&device.addresses).copied().collect();
    device.addresses = candidate_addresses(&dedup(all));
    device
}

fn dedup(addresses: Vec<IpAddr>) -> Vec<IpAddr> {
    let mut unique = Vec::with_capacity(addresses.len());
    for address in addresses {
        if !unique.contains(&address) {
            unique.push(address);
        }
    }
    unique
}

impl crate::util::peers::PeerDirectory for PeerRegistry {
    fn peers(&self) -> Vec<DeviceInfo> {
        self.snapshot()
    }
}
