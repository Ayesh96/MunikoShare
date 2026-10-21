//! End-to-end board tests over real loopback HTTPS servers.

use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::client::BoardClient;
use super::*;
use crate::constants::testing::LOOPBACK;
use crate::device::DeviceInfo;
use crate::events::EventBus;
use crate::identity::Identity;
use crate::lifecycle::Lifecycle;
use crate::server::{HttpServer, RouteProvider};
use crate::util::peers::PeerDirectory;
use crate::util::test_support::shared_test_identity;

pub(super) const WAIT: Duration = Duration::from_secs(10);

#[derive(Default)]
pub(super) struct FakePeers(pub(super) Mutex<Vec<DeviceInfo>>);

impl PeerDirectory for FakePeers {
    fn peers(&self) -> Vec<DeviceInfo> {
        self.0.lock().unwrap().clone()
    }
}

pub(super) struct Node {
    pub(super) identity: Arc<Identity>,
    pub(super) service: BoardService,
    pub(super) server: HttpServer,
    pub(super) runtime: BoardRuntime,
    pub(super) peers: Arc<FakePeers>,
    pub(super) info: DeviceInfo,
}

pub(super) fn fast_config() -> BoardConfig {
    BoardConfig { sync_interval: Duration::from_millis(300), discovery_grace: Duration::from_millis(600), ..BoardConfig::default() }
}

async fn node(name: &str) -> Node {
    node_with(name, fast_config()).await
}

pub(super) async fn node_with(name: &str, config: BoardConfig) -> Node {
    let identity = shared_test_identity();
    let peers = Arc::new(FakePeers::default());
    let service = BoardService::new(identity.clone(), Arc::new(MemoryBoardStore::new()), EventBus::new(), peers.clone(), name.into(), config);
    let providers: Vec<Arc<dyn RouteProvider>> = vec![Arc::new(service.routes())];
    let mut server = HttpServer::new(identity.clone(), providers);
    server.start().await.unwrap();
    let info = DeviceInfo {
        id: identity.device_id().to_string(),
        name: name.into(),
        os: "test".into(),
        addresses: vec![LOOPBACK.parse::<IpAddr>().unwrap()],
        port: server.port().unwrap(),
        protocol_version: 1,
    };
    let runtime = service.runtime();
    Node { identity, service, server, runtime, peers, info }
}

pub(super) fn connect(nodes: &[&Node]) {
    for node in nodes {
        let others = nodes.iter().filter(|other| other.info.id != node.info.id).map(|other| other.info.clone()).collect();
        *node.peers.0.lock().unwrap() = others;
    }
}

pub(super) async fn wait_for(what: &str, mut condition: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + WAIT;
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn texts(node: &Node) -> Vec<String> {
    node.service
        .items()
        .unwrap()
        .into_iter()
        .filter_map(|view| match view.kind {
            ViewKind::Text { text } => Some(text),
            _ => None,
        })
        .collect()
}

pub(super) async fn shutdown(nodes: Vec<Node>) {
    for mut node in nodes {
        node.runtime.stop().await.unwrap();
        node.server.stop().await.unwrap();
    }
}

async fn trio() -> (Node, Node, Node) {
    let (a, b, c) = (node("a").await, node("b").await, node("c").await);
    connect(&[&a, &b, &c]);
    a.service.create_board("Office", BoardMode::Open, None).unwrap();
    b.service.join_board(&a.info.id, None).await.unwrap();
    c.service.join_board(&a.info.id, None).await.unwrap();
    (a, b, c)
}

#[tokio::test]
async fn open_join_copies_board_and_members() {
    let (a, b) = (node("a").await, node("b").await);
    connect(&[&a, &b]);
    let created = a.service.create_board("Home", BoardMode::Open, None).unwrap();
    a.service.post_text("before join").unwrap();
    let joined = b.service.join_board(&a.info.id, None).await.unwrap();
    assert_eq!(joined.id, created.id);
    assert_eq!(texts(&b), vec!["before join"]);
    let info = BoardClient::new(b.identity.clone()).info(&a.info).await.unwrap().unwrap();
    assert_eq!(info.member_count, 2);
    assert_eq!(b.service.items().unwrap()[0].author_name, "a");
    shutdown(vec![a, b]).await;
}

#[tokio::test]
async fn protected_join_needs_the_right_code() {
    let (a, b) = (node("a").await, node("b").await);
    connect(&[&a, &b]);
    a.service.create_board("Cafe", BoardMode::Protected, Some("s3cret")).unwrap();
    assert!(b.service.join_board(&a.info.id, None).await.is_err());
    assert!(b.service.join_board(&a.info.id, Some("wrong")).await.is_err());
    assert!(b.service.board().unwrap().is_none());
    b.service.join_board(&a.info.id, Some("s3cret")).await.unwrap();
    assert!(b.service.board().unwrap().is_some());
    shutdown(vec![a, b]).await;
}

#[tokio::test]
async fn non_member_cannot_push_and_tampered_items_are_rejected() {
    let (a, b) = (node("a").await, node("b").await);
    connect(&[&a, &b]);
    a.service.create_board("Home", BoardMode::Open, None).unwrap();
    let outsider = shared_test_identity();
    let board_id = a.service.board().unwrap().unwrap().id;
    let signer = super::signing::ItemSigner::new(outsider.clone());
    let item = signer.sign(&board_id, ItemKind::Text { text: "hi".into() }).unwrap();
    let client = BoardClient::new(outsider);
    assert!(client.push(&a.info, vec![item]).await.is_err());

    b.service.join_board(&a.info.id, None).await.unwrap();
    let mut forged = super::signing::ItemSigner::new(b.identity.clone()).sign(&board_id, ItemKind::Text { text: "real".into() }).unwrap();
    forged.kind = ItemKind::Text { text: "forged".into() };
    let reply = BoardClient::new(b.identity.clone()).push(&a.info, vec![forged]).await.unwrap();
    assert_eq!((reply.added, reply.rejected), (0, 1));
    assert!(texts(&a).is_empty());
    shutdown(vec![a, b]).await;
}

#[tokio::test]
async fn posts_reach_every_member_by_push() {
    let (mut a, mut b, mut c) = trio().await;
    for node in [&mut a, &mut b, &mut c] {
        node.runtime.start().await.unwrap();
    }
    b.service.post_text("from b").unwrap();
    wait_for("push to a and c", || texts(&a) == ["from b"] && texts(&c) == ["from b"]).await;
    shutdown(vec![a, b, c]).await;
}

#[tokio::test]
async fn offline_member_catches_up_by_digest_pull() {
    let (mut a, mut b, _c) = trio().await;
    a.runtime.start().await.unwrap();
    a.peers.0.lock().unwrap().retain(|peer| peer.id != b.info.id);
    a.service.post_text("while b was away").unwrap();
    a.service.post_text("second").unwrap();
    assert!(texts(&b).is_empty());
    b.runtime.start().await.unwrap();
    wait_for("b to catch up", || texts(&b).len() == 2).await;
    shutdown(vec![a, b, _c]).await;
}

#[tokio::test]
async fn tombstone_hides_item_everywhere_and_only_author_can_delete() {
    let (mut a, mut b, c) = trio().await;
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    let posted = a.service.post_text("oops").unwrap();
    wait_for("b to get the item", || texts(&b) == ["oops"]).await;
    assert!(b.service.delete_item(&posted.id).is_err());
    a.service.delete_item(&posted.id).unwrap();
    wait_for("b to hide it", || texts(&b).is_empty()).await;
    assert!(texts(&a).is_empty());
    assert!(b.server.port().is_some());
    shutdown(vec![a, b, c]).await;
}

#[tokio::test]
async fn two_boards_are_reported_not_merged() {
    let (mut a, mut b) = (node("a").await, node("b").await);
    connect(&[&a, &b]);
    let first = a.service.create_board("One", BoardMode::Open, None).unwrap();
    let second = b.service.create_board("Two", BoardMode::Open, None).unwrap();
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    wait_for("conflict", || !a.service.conflicts().is_empty() && !b.service.conflicts().is_empty()).await;
    assert_eq!(a.service.conflicts()[0].info.board.id, second.id);
    assert_eq!(a.service.board().unwrap().unwrap().id, first.id);
    assert_eq!(b.service.board().unwrap().unwrap().id, second.id);
    shutdown(vec![a, b]).await;
}

#[tokio::test]
async fn bootstrap_creates_then_later_device_joins() {
    let (mut a, mut b) = (node("a").await, node("b").await);
    a.runtime.start().await.unwrap();
    wait_for("a to create a board", || a.service.board().unwrap().is_some()).await;
    connect(&[&a, &b]);
    b.runtime.start().await.unwrap();
    wait_for("b to join", || b.service.board().unwrap().is_some()).await;
    assert_eq!(a.service.board().unwrap().unwrap().id, b.service.board().unwrap().unwrap().id);
    b.service.leave_board().unwrap();
    assert!(b.service.board().unwrap().is_none());
    shutdown(vec![a, b]).await;
}

#[test]
fn retention_prunes_old_items_but_keeps_members() {
    use super::replica::Replica;
    let store = Arc::new(MemoryBoardStore::new());
    let identity = shared_test_identity();
    let signer = super::signing::ItemSigner::new(identity.clone());
    let record = super::model::BoardRecord { board: Board { id: "b".into(), name: "n".into(), mode: BoardMode::Open, created_at: 0 }, join_secret: None };
    store.save_board(&record).unwrap();
    let replica = Replica::new(store.clone(), Duration::from_secs(3600));
    let member = signer.sign("b", ItemKind::Member { device_id: identity.device_id().into(), name: "me".into(), cert_der: identity.cert_der().to_vec() }).unwrap();
    let text = signer.sign("b", ItemKind::Text { text: "old".into() }).unwrap();
    let now = member.created_at;
    assert_eq!(replica.accept(vec![member, text.clone()], now).unwrap().added.len(), 2);
    assert_eq!(replica.prune(now + 7200).unwrap(), 1);
    assert!(store.get(&text.id).unwrap().is_none());
    assert_eq!(store.all().unwrap().len(), 1);
    let late = replica.accept(vec![text], now + 7200).unwrap();
    assert!(late.added.is_empty() && late.rejected == 0);
}
