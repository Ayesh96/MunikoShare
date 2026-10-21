use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::broadcast::Receiver;

use super::broadcast::{BroadcastConfig, BroadcastService};
use super::registry::PeerRegistry;
use super::*;
use crate::events::NodeEvent;
use crate::constants::app::PROTOCOL_VERSION;
use crate::constants::discovery::{TEST_BROADCAST_INTERVAL, TEST_BROADCAST_PORT, TEST_PEER_TTL, TEST_PEER_WAIT_MS, TEST_POLL_INTERVAL_MS};

fn device(id: &str, port: u16) -> DeviceInfo {
    DeviceInfo {
        id: id.repeat(64),
        name: format!("device-{id}"),
        os: "test".into(),
        addresses: Vec::new(),
        port,
        protocol_version: PROTOCOL_VERSION,
    }
}

fn at(device: DeviceInfo, ip: &str) -> DeviceInfo {
    DeviceInfo { addresses: vec![ip.parse().unwrap()], ..device }
}

fn registry() -> (Arc<PeerRegistry>, Receiver<NodeEvent>) {
    let bus = EventBus::new();
    let events = bus.subscribe();
    (Arc::new(PeerRegistry::new(device("z", 1).id, bus)), events)
}

async fn wait_for_peer(peers: impl Fn() -> Vec<DeviceInfo>, peer_id: &str) -> bool {
    for _ in 0..(TEST_PEER_WAIT_MS / TEST_POLL_INTERVAL_MS) {
        if peers().iter().any(|peer| peer.id == peer_id) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(TEST_POLL_INTERVAL_MS)).await;
    }
    false
}

#[tokio::test]
async fn two_services_discover_each_other() {
    let (a, b) = (device("a", 41001), device("b", 41002));
    let mut first = DiscoveryService::new(a.clone(), EventBus::new());
    let mut second = DiscoveryService::new(b.clone(), EventBus::new());
    first.start().await.unwrap();
    second.start().await.unwrap();

    assert!(wait_for_peer(|| first.peers(), &b.id).await);
    assert!(wait_for_peer(|| second.peers(), &a.id).await);
    let seen = first.peers().into_iter().find(|peer| peer.id == b.id).unwrap();
    assert_eq!((seen.port, seen.name), (41002, b.name));
    assert!(first.peers().iter().all(|peer| peer.id != a.id));

    first.stop().await.unwrap();
    second.stop().await.unwrap();
    assert_eq!(first.state(), LifecycleState::Stopped);
}

#[tokio::test]
async fn starting_twice_is_invalid_state() {
    let mut service = DiscoveryService::new(device("c", 41003), EventBus::new());
    service.start().await.unwrap();
    assert!(matches!(service.start().await, Err(MunikoError::InvalidState(_))));
    service.stop().await.unwrap();
}

#[tokio::test]
async fn manual_peer_is_published_listed_and_removable() {
    let bus = EventBus::new();
    let mut events = bus.subscribe();
    let service = DiscoveryService::new(device("a", 1), bus);
    let peer = at(device("m", 7), "192.168.1.50");

    service.add_manual_peer(peer.clone());
    assert!(matches!(events.recv().await.unwrap(), NodeEvent::PeerAdded { device } if device == peer));
    assert_eq!(service.peers(), vec![peer.clone()]);

    service.remove_manual_peer(&peer.id);
    assert!(matches!(events.recv().await.unwrap(), NodeEvent::PeerLost { device_id } if device_id == peer.id));
    assert!(service.peers().is_empty());
}

#[test]
fn manual_peer_survives_mdns_loss_and_merges_addresses() {
    let (registry, _events) = registry();
    let peer = device("m", 7);
    registry.add_manual(at(peer.clone(), "192.168.1.50"));
    registry.observe_mdns("m._muniko._tcp.local.".into(), at(peer.clone(), "10.0.0.5"));
    registry.mdns_removed("m._muniko._tcp.local.");

    let kept = registry.snapshot();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].addresses.len(), 2);
}

#[test]
fn broadcast_peer_expires_after_ttl_unless_vouched_for() {
    let (registry, mut events) = registry();
    let (loose, manual) = (at(device("l", 1), "192.168.1.2"), at(device("m", 2), "192.168.1.3"));
    let start = Instant::now();
    registry.observe_broadcast(loose.clone(), start);
    registry.add_manual(manual.clone());
    registry.observe_broadcast(manual.clone(), start);

    registry.expire_broadcast(TEST_PEER_TTL, start + TEST_PEER_TTL / 2);
    assert_eq!(registry.snapshot().len(), 2);
    registry.expire_broadcast(TEST_PEER_TTL, start + TEST_PEER_TTL * 2);
    assert_eq!(registry.snapshot(), vec![manual]);

    let mut lost = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let NodeEvent::PeerLost { device_id } = event {
            lost.push(device_id);
        }
    }
    assert_eq!(lost, vec![loose.id]);
}

#[test]
fn repeated_broadcast_sightings_publish_found_once() {
    let (registry, mut events) = registry();
    let peer = at(device("b", 3), "192.168.1.4");
    registry.observe_broadcast(peer.clone(), Instant::now());
    registry.observe_broadcast(peer, Instant::now());
    assert!(matches!(events.try_recv(), Ok(NodeEvent::PeerFound { .. })));
    assert!(events.try_recv().is_err());
}

#[tokio::test]
async fn broadcast_components_discover_each_other_and_ignore_themselves() {
    // GitHub's macOS runners deny LAN broadcast to unsigned test binaries; other OSes still run it.
    if cfg!(target_os = "macos") && std::env::var_os("GITHUB_ACTIONS").is_some() {
        return;
    }
    let config = BroadcastConfig { port: TEST_BROADCAST_PORT, interval: TEST_BROADCAST_INTERVAL, peer_ttl: TEST_PEER_TTL };
    let (a, b) = (device("a", 41011), device("b", 41012));
    let ((reg_a, _ea), (reg_b, _eb)) = (registry(), registry());
    let mut first = BroadcastService::new(a.clone(), reg_a.clone(), config);
    let mut second = BroadcastService::new(b.clone(), reg_b.clone(), config);
    first.start().await.unwrap();
    second.start().await.unwrap();

    assert!(wait_for_peer(|| reg_a.snapshot(), &b.id).await);
    assert!(wait_for_peer(|| reg_b.snapshot(), &a.id).await);
    assert!(reg_a.snapshot().iter().all(|peer| peer.id != a.id));

    second.stop().await.unwrap();
    first.stop().await.unwrap();
}
