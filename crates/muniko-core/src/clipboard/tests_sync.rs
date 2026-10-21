//! Watcher, loop prevention, sensitivity and history tests over loopback servers.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast::Receiver;

use super::tests_support::{test_image, FakeClipboard, FakeProbe, FixedPeers, TestNode};
use super::{ClipboardContent, ClipboardHistory, ClipboardWatcher, HistorySource};
use crate::constants::clipboard::MAX_IMAGE_PNG_BYTES;
use crate::events::NodeEvent;
use crate::lifecycle::Lifecycle;

const TICK: Duration = Duration::from_millis(40);
const SETTLE: Duration = Duration::from_millis(600);

fn text(value: &str) -> ClipboardContent {
    ClipboardContent::Text(value.to_string())
}

fn watcher(node: &TestNode, peers: Vec<&TestNode>, probe: Arc<FakeProbe>) -> ClipboardWatcher {
    let directory = Arc::new(FixedPeers::default());
    *directory.0.lock().unwrap() = peers.iter().map(|peer| peer.info()).collect();
    ClipboardWatcher::new(node.identity.clone(), node.clipboard.clone(), node.trust.clone(), directory, node.events.clone(), node.shared.clone())
        .with_probe(probe)
        .with_interval(TICK)
}

fn sent_to(events: &mut Receiver<NodeEvent>) -> Vec<String> {
    let mut sent = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let NodeEvent::ClipboardSent { to_device_id } = event {
            sent.push(to_device_id);
        }
    }
    sent
}

#[tokio::test]
async fn watcher_sends_change_to_sync_enabled_peer_only() {
    let mut a = TestNode::start(FakeClipboard::with(text("old"))).await;
    let mut on = TestNode::start(FakeClipboard::empty()).await;
    let mut off = TestNode::start(FakeClipboard::empty()).await;
    on.trust_device(&a, false);
    off.trust_device(&a, false);
    a.trust_device(&on, true);
    a.trust_device(&off, false);
    let mut events = a.events.subscribe();

    let mut w = watcher(&a, vec![&on, &off], Arc::default());
    w.start().await.unwrap();
    tokio::time::sleep(SETTLE).await;
    assert_eq!(on.clipboard.writes(), 0, "baseline must not be sent");
    a.clipboard.user_copy(text("new"));
    tokio::time::sleep(SETTLE).await;
    w.stop().await.unwrap();

    assert_eq!(on.clipboard.text().as_deref(), Some("new"));
    assert_eq!(off.clipboard.writes(), 0);
    assert_eq!(sent_to(&mut events), vec![on.identity.device_id().to_string()]);
    for node in [&mut a, &mut on, &mut off] {
        node.server.stop().await.unwrap();
    }
}

#[tokio::test]
async fn received_content_does_not_bounce_back() {
    let mut a = TestNode::start(FakeClipboard::with(text("old"))).await;
    let mut b = TestNode::start(FakeClipboard::with(text("old"))).await;
    a.trust_device(&b, true);
    b.trust_device(&a, true);
    let (mut a_events, mut b_events) = (a.events.subscribe(), b.events.subscribe());
    let (mut wa, mut wb) = (watcher(&a, vec![&b], Arc::default()), watcher(&b, vec![&a], Arc::default()));
    wa.start().await.unwrap();
    wb.start().await.unwrap();
    tokio::time::sleep(SETTLE).await;

    a.clipboard.user_copy(text("shared"));
    tokio::time::sleep(SETTLE * 3).await;
    wa.stop().await.unwrap();
    wb.stop().await.unwrap();

    assert_eq!(b.clipboard.text().as_deref(), Some("shared"));
    assert_eq!(b.clipboard.writes(), 1);
    assert_eq!(a.clipboard.writes(), 0);
    assert_eq!(sent_to(&mut a_events).len(), 1);
    assert!(sent_to(&mut b_events).is_empty());
    a.server.stop().await.unwrap();
    b.server.stop().await.unwrap();
}

#[tokio::test]
async fn image_change_syncs_and_publishes_image_event() {
    let mut a = TestNode::start(FakeClipboard::with(text("old"))).await;
    let mut b = TestNode::start(FakeClipboard::empty()).await;
    a.trust_device(&b, true);
    b.trust_device(&a, false);
    let mut b_events = b.events.subscribe();
    let mut w = watcher(&a, vec![&b], Arc::default());
    w.start().await.unwrap();
    tokio::time::sleep(SETTLE).await;
    let image = test_image(8, 6);
    a.clipboard.user_copy(image.clone());
    tokio::time::sleep(SETTLE).await;
    w.stop().await.unwrap();

    assert_eq!(b.clipboard.current(), Some(image));
    assert!(matches!(b_events.try_recv(), Ok(NodeEvent::ClipboardImageReceived { width: 8, height: 6, .. })));
    a.server.stop().await.unwrap();
    b.server.stop().await.unwrap();
}

#[tokio::test]
async fn sensitive_items_are_not_synced() {
    let mut a = TestNode::start(FakeClipboard::with(text("old"))).await;
    let mut b = TestNode::start(FakeClipboard::empty()).await;
    a.trust_device(&b, true);
    b.trust_device(&a, false);
    let probe = Arc::new(FakeProbe::default());
    let mut w = watcher(&a, vec![&b], probe.clone());
    w.start().await.unwrap();
    tokio::time::sleep(SETTLE).await;
    probe.0.store(true, Ordering::SeqCst);
    a.clipboard.user_copy(text("hunter2"));
    tokio::time::sleep(SETTLE).await;
    w.stop().await.unwrap();

    assert_eq!(b.clipboard.writes(), 0);
    assert!(a.shared.history.list().is_empty());
    a.server.stop().await.unwrap();
    b.server.stop().await.unwrap();
}

#[tokio::test]
async fn oversized_image_is_skipped_by_watcher() {
    let mut a = TestNode::start(FakeClipboard::with(text("old"))).await;
    let mut b = TestNode::start(FakeClipboard::empty()).await;
    a.trust_device(&b, true);
    b.trust_device(&a, false);
    let mut w = watcher(&a, vec![&b], Arc::default());
    w.start().await.unwrap();
    tokio::time::sleep(SETTLE).await;
    let huge = ClipboardContent::Image { width: 1, height: 1, png: vec![0; MAX_IMAGE_PNG_BYTES + 1] };
    a.clipboard.user_copy(huge);
    tokio::time::sleep(SETTLE).await;
    w.stop().await.unwrap();

    assert_eq!(b.clipboard.writes(), 0);
    a.server.stop().await.unwrap();
    b.server.stop().await.unwrap();
}

#[tokio::test]
async fn watcher_skips_reading_when_no_peer_syncs() {
    let mut a = TestNode::start(FakeClipboard::with(text("old"))).await;
    let b = TestNode::start(FakeClipboard::empty()).await;
    a.trust_device(&b, false);
    let mut w = watcher(&a, vec![&b], Arc::default());
    w.start().await.unwrap();
    a.clipboard.user_copy(text("x"));
    tokio::time::sleep(SETTLE).await;
    w.stop().await.unwrap();
    assert!(a.shared.history.list().is_empty());
    a.server.stop().await.unwrap();
}

#[tokio::test]
async fn history_is_bounded_and_copy_back_writes_clipboard() {
    let clipboard = FakeClipboard::empty();
    let history = ClipboardHistory::with_capacity(clipboard.clone(), 3);
    for n in 0..5 {
        history.record(HistorySource::Local, text(&format!("item {n}")));
    }
    let list = history.list();
    assert_eq!(list.len(), 3);
    let newest = &list[0];
    assert_eq!(history.text(&newest.id).as_deref(), Some("item 4"));

    let oldest = list[2].id.clone();
    history.copy_back(&oldest).await.unwrap();
    assert_eq!(clipboard.text().as_deref(), Some("item 2"));
    assert!(history.copy_back("missing").await.is_err());

    history.record(HistorySource::Local, text("item 2"));
    assert_eq!(history.list().len(), 3, "duplicates collapse");
    history.clear();
    assert!(history.list().is_empty());
}

#[tokio::test]
async fn history_pins_survive_eviction_and_images_expose_data_url() {
    let history = ClipboardHistory::with_capacity(FakeClipboard::empty(), 2);
    history.record(HistorySource::Local, text("keep"));
    let keep = history.list()[0].id.clone();
    assert!(history.set_pinned(&keep, true));
    history.record(HistorySource::Local, text("b"));
    history.record(HistorySource::Local, test_image(2, 2));
    let list = history.list();
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|item| item.id == keep && item.pinned));
    let image_id = list.iter().find(|item| item.id != keep).unwrap().id.clone();
    assert!(history.image_data_url(&image_id).unwrap().starts_with("data:image/png;base64,"));
    assert!(history.text(&image_id).is_none());
}
