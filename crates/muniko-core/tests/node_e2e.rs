use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use muniko_core::history::HistoryKind;
use muniko_core::{Lifecycle, MunikoNode, NodeConfig, NodeEvent};
use tempfile::TempDir;
use tokio::sync::broadcast::Receiver;

const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);
const EVENT_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(200);
const FILE_NAME: &str = "hello.txt";
const FILE_CONTENT: &[u8] = b"hello from node a";
const HISTORY_LIMIT: usize = 10;
const LOOPBACK: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

async fn make_node(root: &TempDir, name: &str) -> MunikoNode {
    let config = NodeConfig {
        device_name: name.to_string(),
        data_dir: root.path().join(name).join("data"),
        download_dir: root.path().join(name).join("downloads"),
    };
    MunikoNode::new(config).await.expect("node created")
}

// Waits until the peer is known with a usable address, not just seen.
async fn wait_for_peer(node: &MunikoNode, peer_id: &str) {
    let deadline = tokio::time::Instant::now() + DISCOVERY_TIMEOUT;
    while !node.peers().iter().any(|peer| peer.id == peer_id && !peer.addresses.is_empty()) {
        assert!(tokio::time::Instant::now() < deadline, "peer not discovered in time");
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn next_matching<T>(events: &mut Receiver<NodeEvent>, pick: impl Fn(NodeEvent) -> Option<T>) -> T {
    let wait = async {
        loop {
            if let Some(found) = pick(events.recv().await.expect("event channel open")) {
                return found;
            }
        }
    };
    tokio::time::timeout(EVENT_TIMEOUT, wait).await.expect("expected event in time")
}

#[tokio::test]
async fn two_nodes_pair_then_transfer_a_file() {
    let root = TempDir::new().unwrap();
    let mut sender = make_node(&root, "node-a").await;
    let mut receiver = make_node(&root, "node-b").await;
    let mut receiver_events = receiver.subscribe();
    sender.start().await.unwrap();
    receiver.start().await.unwrap();

    let sender_id = sender.local_device().id;
    let receiver_id = receiver.local_device().id;
    wait_for_peer(&sender, &receiver_id).await;

    sender.request_pairing(&receiver_id).await.expect("pairing requested");
    let code = next_matching(&mut receiver_events, |event| match event {
        NodeEvent::PairingRequested { code, .. } => Some(code),
        _ => None,
    })
    .await;
    sender.confirm_pairing(&receiver_id, &code).await.expect("pairing confirmed");
    assert!(sender.trusted_devices().iter().any(|device| device.id == receiver_id));
    assert!(receiver.trusted_devices().iter().any(|device| device.id == sender_id));

    let source = root.path().join(FILE_NAME);
    tokio::fs::write(&source, FILE_CONTENT).await.unwrap();
    let send = sender.send_files(&receiver_id, std::slice::from_ref(&source));
    let accept_and_receive = async {
        let offer_id = next_matching(&mut receiver_events, |event| match event {
            NodeEvent::IncomingOffer { offer_id, .. } => Some(offer_id),
            _ => None,
        })
        .await;
        receiver.respond_to_offer(&offer_id, true).expect("offer accepted");
        next_matching(&mut receiver_events, |event| match event {
            NodeEvent::FileReceived { path, from_device_id } => Some((path, from_device_id)),
            _ => None,
        })
        .await
    };
    let (sent, (received_path, from_device_id)) = tokio::join!(send, accept_and_receive);
    sent.expect("files sent");
    assert_eq!(from_device_id, sender_id);
    assert_eq!(tokio::fs::read(&received_path).await.unwrap(), FILE_CONTENT);

    let history_has = |node: &MunikoNode, kind: HistoryKind| node.history(HISTORY_LIMIT).unwrap().iter().any(|entry| entry.kind == kind);
    let deadline = tokio::time::Instant::now() + EVENT_TIMEOUT;
    while !(history_has(&receiver, HistoryKind::FileReceived) && history_has(&sender, HistoryKind::FileSent)) {
        assert!(tokio::time::Instant::now() < deadline, "history not recorded in time");
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    assert!(history_has(&receiver, HistoryKind::Paired));

    sender.stop().await.unwrap();
    receiver.stop().await.unwrap();
}

#[tokio::test]
async fn adds_a_peer_by_ip_address() {
    let root = TempDir::new().unwrap();
    let mut adder = make_node(&root, "node-c").await;
    let mut target = make_node(&root, "node-d").await;
    adder.start().await.unwrap();
    target.start().await.unwrap();

    let expected = target.local_device();
    let device = adder.add_peer_by_address(LOOPBACK, expected.port).await.expect("peer added");
    assert_eq!(device.id, expected.id);
    assert!(adder.peers().iter().any(|peer| peer.id == expected.id));

    adder.stop().await.unwrap();
    target.stop().await.unwrap();
}
