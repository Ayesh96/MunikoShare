//! Board file content over real loopback servers: caching, serving, resume, eviction.

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::client::BoardClient;
use super::tests::{connect, fast_config, node_with, shutdown, wait_for, Node};
use super::*;
use crate::lifecycle::Lifecycle;
use crate::util::test_support::shared_test_identity;

const SMALL: usize = 600;
const LARGE: usize = 4000;
const LIMIT: u64 = 1024;

fn config_in(root: &Path, name: &str) -> BoardConfig {
    let base = root.join(name);
    BoardConfig { files_dir: base.join("cache"), download_dir: base.join("dl"), auto_cache_limit: LIMIT, ..fast_config() }
}

async fn files_node(root: &Path, name: &str, tweak: impl FnOnce(&mut BoardConfig)) -> Node {
    let mut config = config_in(root, name);
    tweak(&mut config);
    node_with(name, config).await
}

async fn files_trio(root: &Path) -> (Node, Node, Node) {
    let (a, b, c) = (files_node(root, "a", |_| {}).await, files_node(root, "b", |_| {}).await, files_node(root, "c", |_| {}).await);
    connect(&[&a, &b, &c]);
    a.service.create_board("Office", BoardMode::Open, None).unwrap();
    b.service.join_board(&a.info.id, None).await.unwrap();
    c.service.join_board(&a.info.id, None).await.unwrap();
    (a, b, c)
}

fn content(len: usize, seed: u8) -> Vec<u8> {
    (0..len).map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed)).collect()
}

fn source(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let dir = root.join("src");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn file_view(node: &Node, name: &str) -> Option<BoardItemView> {
    node.service.items().unwrap().into_iter().find(|view| matches!(&view.kind, ViewKind::File { name: n, .. } if n == name))
}

fn sha_of(view: &BoardItemView) -> String {
    match &view.kind {
        ViewKind::File { sha256, .. } => sha256.clone(),
        _ => unreachable!(),
    }
}

fn held(node: &Node, name: &str) -> bool {
    file_view(node, name).is_some_and(|view| view.held_locally)
}

fn cache_path(root: &Path, node: &str, sha: &str) -> PathBuf {
    root.join(node).join("cache").join(sha)
}

async fn take_offline(node: &mut Node) {
    node.runtime.stop().await.unwrap();
    node.server.stop().await.unwrap();
}

#[tokio::test]
async fn small_file_is_auto_cached_from_a_non_author_holder() {
    let root = tempfile::tempdir().unwrap();
    let (mut a, mut b, mut c) = files_trio(root.path()).await;
    let bytes = content(SMALL, 1);
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    let posted = a.service.post_file(&source(root.path(), "note.bin", &bytes)).await.unwrap();
    assert!(posted.held_locally && posted.available);
    wait_for("b to cache the file", || held(&b, "note.bin")).await;

    take_offline(&mut a).await;
    *b.peers.0.lock().unwrap() = vec![c.info.clone()];
    *c.peers.0.lock().unwrap() = vec![b.info.clone()];
    c.runtime.start().await.unwrap();
    wait_for("c to cache the file from b", || held(&c, "note.bin")).await;
    assert_eq!(std::fs::read(cache_path(root.path(), "c", &sha_of(&posted))).unwrap(), bytes);
    shutdown(vec![b, c]).await;
}

#[tokio::test]
async fn large_file_is_fetched_only_on_demand_and_exports_with_unique_names() {
    let root = tempfile::tempdir().unwrap();
    let (mut a, mut b, c) = files_trio(root.path()).await;
    let bytes = content(LARGE, 2);
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    let posted = a.service.post_file(&source(root.path(), "big.bin", &bytes)).await.unwrap();
    wait_for("b to see an available file", || file_view(&b, "big.bin").is_some_and(|v| v.available)).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(!held(&b, "big.bin"), "large files must not be auto-cached");

    let first = b.service.download_file(&posted.id).await.unwrap();
    assert_eq!(std::fs::read(&first).unwrap(), bytes);
    assert!(held(&b, "big.bin"));
    let second = b.service.download_file(&posted.id).await.unwrap();
    assert_ne!(first, second);
    assert_eq!(second.file_name().unwrap().to_str().unwrap(), "big (1).bin");

    take_offline(&mut a).await;
    *b.peers.0.lock().unwrap() = Vec::new();
    wait_for("c-less view still available from the local cache", || file_view(&b, "big.bin").is_some_and(|v| v.held_locally && v.available)).await;
    shutdown(vec![b, c]).await;
}

#[tokio::test]
async fn file_shows_unavailable_when_no_online_device_holds_it() {
    let root = tempfile::tempdir().unwrap();
    let (mut a, mut b, c) = files_trio(root.path()).await;
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    let posted = a.service.post_file(&source(root.path(), "gone.bin", &content(LARGE, 3))).await.unwrap();
    wait_for("b to see it available", || file_view(&b, "gone.bin").is_some_and(|v| v.available)).await;
    take_offline(&mut a).await;
    *b.peers.0.lock().unwrap() = Vec::new();
    let view = file_view(&b, "gone.bin").unwrap();
    assert!(!view.held_locally && !view.available);
    assert!(b.service.download_file(&posted.id).await.is_err());
    shutdown(vec![b, c]).await;
}

#[tokio::test]
async fn range_requests_resume_and_bad_ranges_fail() {
    let root = tempfile::tempdir().unwrap();
    let (a, b, c) = files_trio(root.path()).await;
    let bytes = content(LARGE, 4);
    let posted = a.service.post_file(&source(root.path(), "r.bin", &bytes)).await.unwrap();
    let sha = sha_of(&posted);
    let client = BoardClient::new(b.identity.clone());

    let (whole, resumed) = client.open_file(&a.info, &sha, 0).await.unwrap();
    assert!(!resumed);
    assert_eq!(whole.bytes().await.unwrap().as_ref(), &bytes[..]);
    let (tail, resumed) = client.open_file(&a.info, &sha, 1000).await.unwrap();
    assert!(resumed);
    assert_eq!(tail.headers()["content-range"].to_str().unwrap(), format!("bytes 1000-{}/{LARGE}", LARGE - 1));
    assert_eq!(tail.bytes().await.unwrap().as_ref(), &bytes[1000..]);
    assert!(client.open_file(&a.info, &sha, LARGE as u64).await.is_err());
    assert!(client.open_file(&a.info, &"0".repeat(64), 0).await.is_err());

    // A leftover .part from an interrupted download is continued, not restarted.
    let part = root.path().join("b").join("cache").join(format!("{sha}.part"));
    std::fs::create_dir_all(part.parent().unwrap()).unwrap();
    std::fs::write(&part, &bytes[..1500]).unwrap();
    let mut b = b;
    b.runtime.start().await.unwrap();
    wait_for("b to know the item", || file_view(&b, "r.bin").is_some_and(|v| v.available)).await;
    let exported = b.service.download_file(&posted.id).await.unwrap();
    assert_eq!(std::fs::read(exported).unwrap(), bytes);
    assert!(!part.exists());
    shutdown(vec![a, b, c]).await;
}

#[tokio::test]
async fn non_member_gets_403_on_the_file_route() {
    let root = tempfile::tempdir().unwrap();
    let (a, b, c) = files_trio(root.path()).await;
    let posted = a.service.post_file(&source(root.path(), "s.bin", &content(SMALL, 5))).await.unwrap();
    let outsider = BoardClient::new(shared_test_identity());
    let error = outsider.open_file(&a.info, &sha_of(&posted), 0).await.unwrap_err();
    assert!(error.to_string().contains("403"), "{error}");
    shutdown(vec![a, b, c]).await;
}

#[tokio::test]
async fn lying_holder_is_rejected_and_the_next_holder_is_tried() {
    let root = tempfile::tempdir().unwrap();
    let (mut a, mut b, mut c) = (
        files_node(root.path(), "a", |_| {}).await,
        files_node(root.path(), "b", |_| {}).await,
        files_node(root.path(), "c", |config| config.auto_cache_limit = 0).await,
    );
    connect(&[&a, &b, &c]);
    a.service.create_board("Office", BoardMode::Open, None).unwrap();
    b.service.join_board(&a.info.id, None).await.unwrap();
    c.service.join_board(&a.info.id, None).await.unwrap();
    let bytes = content(SMALL, 6);
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    let posted = a.service.post_file(&source(root.path(), "liar.bin", &bytes)).await.unwrap();
    wait_for("b to cache", || held(&b, "liar.bin")).await;
    std::fs::write(cache_path(root.path(), "b", &sha_of(&posted)), content(SMALL, 99)).unwrap();

    // b is listed first so it is tried first.
    *c.peers.0.lock().unwrap() = vec![b.info.clone(), a.info.clone()];
    c.runtime.start().await.unwrap();
    wait_for("c to see the item", || file_view(&c, "liar.bin").is_some()).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let exported = c.service.download_file(&posted.id).await.unwrap();
    assert_eq!(std::fs::read(exported).unwrap(), bytes);
    assert_eq!(std::fs::read(cache_path(root.path(), "c", &sha_of(&posted))).unwrap(), bytes);
    take_offline(&mut a).await;
    shutdown(vec![b, c]).await;
}

#[tokio::test]
async fn cache_cap_evicts_least_recently_used_foreign_files_but_never_own() {
    let root = tempfile::tempdir().unwrap();
    let (mut a, mut b, c) = (
        files_node(root.path(), "a", |config| config.cache_cap = 1000).await,
        files_node(root.path(), "b", |config| {
            config.auto_cache_limit = 0;
            config.cache_cap = 3000;
        })
        .await,
        files_node(root.path(), "c", |_| {}).await,
    );
    connect(&[&a, &b, &c]);
    a.service.create_board("Office", BoardMode::Open, None).unwrap();
    b.service.join_board(&a.info.id, None).await.unwrap();
    c.service.join_board(&a.info.id, None).await.unwrap();
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    let mut ids = Vec::new();
    for (index, name) in ["one.bin", "two.bin", "three.bin"].into_iter().enumerate() {
        ids.push(a.service.post_file(&source(root.path(), name, &content(1500, index as u8 + 10))).await.unwrap().id);
    }
    wait_for("b to see all files", || b.service.items().unwrap().iter().filter(|v| v.available).count() == 3).await;
    for id in &ids {
        b.service.download_file(id).await.unwrap();
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    assert!(!held(&b, "one.bin"), "oldest foreign file is evicted");
    assert!(held(&b, "two.bin") && held(&b, "three.bin"));
    assert!(["one.bin", "two.bin", "three.bin"].iter().all(|name| held(&a, name)), "own files are never evicted");
    shutdown(vec![a, b, c]).await;
}

#[tokio::test]
async fn keeper_caches_large_files_and_the_setting_sticks() {
    let root = tempfile::tempdir().unwrap();
    let (mut a, mut b, mut c) = files_trio(root.path()).await;
    b.service.set_keeper(true).unwrap();
    assert!(b.service.is_keeper().unwrap() && !c.service.is_keeper().unwrap());
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    c.runtime.start().await.unwrap();
    a.service.post_file(&source(root.path(), "huge.bin", &content(LARGE, 7))).await.unwrap();
    wait_for("keeper to cache the large file", || held(&b, "huge.bin")).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(!held(&c, "huge.bin"));
    shutdown(vec![a, b, c]).await;
}

#[tokio::test]
async fn tombstone_deletes_cached_content_everywhere() {
    let root = tempfile::tempdir().unwrap();
    let (mut a, mut b, c) = files_trio(root.path()).await;
    a.runtime.start().await.unwrap();
    b.runtime.start().await.unwrap();
    let posted = a.service.post_file(&source(root.path(), "del.bin", &content(SMALL, 8))).await.unwrap();
    let sha = sha_of(&posted);
    wait_for("b to cache", || held(&b, "del.bin")).await;
    a.service.delete_item(&posted.id).unwrap();
    assert!(!cache_path(root.path(), "a", &sha).exists());
    wait_for("b to drop the content", || !cache_path(root.path(), "b", &sha).exists()).await;
    shutdown(vec![a, b, c]).await;
}
