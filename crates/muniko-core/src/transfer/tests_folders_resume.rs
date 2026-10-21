//! Folder transfers, path validation, resume and auto-accept over a real socket on loopback.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::tests::{sha_hex, Fixture};
use crate::constants::transfer::*;
use crate::events::{NodeEvent, TransferDirection};
use crate::lifecycle::Lifecycle;
use crate::trust::TrustStore;

fn write(path: &Path, content: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn photos_folder(fx: &Fixture) -> std::path::PathBuf {
    let root = fx.source_dir.path().join("Photos");
    write(&root.join("a.jpg"), b"aaa");
    write(&root.join("trip/b.jpg"), b"bbbb");
    write(&root.join("trip/deep/c.txt"), b"ccccc");
    root
}

fn read(fx: &Fixture, relative: &str) -> Vec<u8> {
    std::fs::read(fx.download_dir.path().join(relative)).unwrap()
}

#[tokio::test]
async fn folder_arrives_with_structure_and_relative_names() {
    let mut fx = Fixture::start().await;
    let root = photos_folder(&fx);
    let loose = fx.source_file("loose.txt", b"x");
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("a.jpg"), root.join("link.jpg")).unwrap();
    fx.answer_next_offer(true);

    fx.client.send_files(&fx.peer(), &[root, loose]).await.unwrap();

    assert_eq!(read(&fx, "Photos/a.jpg"), b"aaa");
    assert_eq!(read(&fx, "Photos/trip/b.jpg"), b"bbbb");
    assert_eq!(read(&fx, "Photos/trip/deep/c.txt"), b"ccccc");
    assert_eq!(fx.downloaded_names(), vec!["Photos".to_string(), "loose.txt".to_string()]);
    assert!(!fx.download_dir.path().join("Photos/link.jpg").exists());
    let mut names = fx
        .drain_events()
        .into_iter()
        .find_map(|e| match e {
            NodeEvent::IncomingOffer { files, .. } => Some(files.into_iter().map(|f| f.name).collect::<Vec<_>>()),
            _ => None,
        })
        .unwrap();
    names.sort();
    assert_eq!(names, vec!["Photos/a.jpg", "Photos/trip/b.jpg", "Photos/trip/deep/c.txt", "loose.txt"]);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn existing_top_folder_gets_a_numbered_twin_for_every_file() {
    let mut fx = Fixture::start().await;
    write(&fx.download_dir.path().join("Photos/old.txt"), b"old");
    let root = photos_folder(&fx);
    fx.answer_next_offer(true);

    fx.client.send_files(&fx.peer(), &[root]).await.unwrap();

    assert_eq!(read(&fx, "Photos/old.txt"), b"old");
    assert_eq!(read(&fx, "Photos (1)/a.jpg"), b"aaa");
    assert_eq!(read(&fx, "Photos (1)/trip/b.jpg"), b"bbbb");
    assert_eq!(read(&fx, "Photos (1)/trip/deep/c.txt"), b"ccccc");
    assert_eq!(fx.downloaded_names(), vec!["Photos".to_string(), "Photos (1)".to_string()]);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn malicious_paths_are_rejected_with_400() {
    let mut fx = Fixture::start().await;
    let client = fx.pinned_http_client();
    for path in ["../x", "/etc/x", "C:\\x", "a//b", "a/../b", "a\\b"] {
        let response = fx.raw_offer(&client, path, b"x", &sha_hex(b"x")).await;
        assert_eq!(response.status(), 400, "{path:?}");
    }
    let too_deep = vec!["d"; MAX_PATH_DEPTH + 1].join("/");
    assert_eq!(fx.raw_offer(&client, &too_deep, b"x", &sha_hex(b"x")).await.status(), 400);
    assert!(fx.downloaded_names().is_empty());
    fx.server.stop().await.unwrap();
}

async fn raw_put_at(fx: &Fixture, id: &str, offset: usize, body: &[u8]) -> reqwest::Response {
    let url = fx.url(&format!("{ROUTE_TRANSFERS}/{id}/files/0?{OFFSET_PARAM}={offset}"));
    fx.pinned_http_client().put(url).body(body.to_vec()).send().await.unwrap()
}

async fn received(fx: &Fixture, id: &str) -> serde_json::Value {
    let url = fx.url(&format!("{ROUTE_TRANSFERS}/{id}/files/0"));
    fx.pinned_http_client().get(url).send().await.unwrap().json().await.unwrap()
}

#[tokio::test]
async fn interrupted_upload_resumes_at_the_stored_offset() {
    let mut fx = Fixture::start().await;
    let content: Vec<u8> = (0..1000u32).map(|i| (i % 253) as u8).collect();
    let id = fx.accepted_transfer("big.bin", &content).await;

    assert_eq!(raw_put_at(&fx, &id, 0, &content[..400]).await.status(), 400);
    assert_eq!(received(&fx, &id).await, serde_json::json!({ "received": 400, "complete": false }));
    assert_eq!(raw_put_at(&fx, &id, 100, &content[100..]).await.status(), 409);
    assert_eq!(raw_put_at(&fx, &id, 0, &content).await.status(), 409);
    assert_eq!(raw_put_at(&fx, &id, 400, &content[400..]).await.status(), 201);

    assert_eq!(read(&fx, "big.bin"), content);
    assert_eq!(fx.downloaded_names(), vec!["big.bin".to_string()]);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn resumed_upload_with_corrupt_tail_fails_the_whole_file_hash() {
    let mut fx = Fixture::start().await;
    let id = fx.accepted_transfer("a.bin", b"0123456789").await;
    assert_eq!(raw_put_at(&fx, &id, 0, b"01234").await.status(), 400);
    assert_eq!(raw_put_at(&fx, &id, 5, b"XXXXX").await.status(), 422);
    assert!(fx.downloaded_names().is_empty());
    fx.server.stop().await.unwrap();
}

// Forwards TCP but cuts the first connection that pushes more than `cut_after` bytes upstream.
async fn cutting_proxy(target_port: u16, cut_after: u64) -> (u16, Arc<AtomicBool>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (forwarded, cut) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicBool::new(false)));
    let cut_flag = cut.clone();
    tokio::spawn(async move {
        loop {
            let (client, _) = listener.accept().await.unwrap();
            let upstream = TcpStream::connect(("127.0.0.1", target_port)).await.unwrap();
            tokio::spawn(relay(client, upstream, forwarded.clone(), cut.clone(), cut_after));
        }
    });
    (port, cut_flag)
}

async fn relay(client: TcpStream, upstream: TcpStream, forwarded: Arc<AtomicU64>, cut: Arc<AtomicBool>, cut_after: u64) {
    let (mut client_read, mut client_write) = client.into_split();
    let (mut upstream_read, mut upstream_write) = upstream.into_split();
    let upload = async {
        let mut buffer = vec![0u8; 16 * 1024];
        loop {
            let read = client_read.read(&mut buffer).await.unwrap_or(0);
            let total = forwarded.fetch_add(read as u64, Ordering::SeqCst) + read as u64;
            if read == 0 || (total > cut_after && !cut.swap(true, Ordering::SeqCst)) {
                break;
            }
            if upstream_write.write_all(&buffer[..read]).await.is_err() {
                break;
            }
        }
    };
    let download = tokio::io::copy(&mut upstream_read, &mut client_write);
    tokio::select! { _ = upload => {}, _ = download => {} }
}

#[tokio::test]
async fn client_resumes_after_the_connection_drops_mid_upload() {
    let mut fx = Fixture::start().await;
    let big: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
    let source = fx.source_file("big.bin", &big);
    let (proxy_port, cut) = cutting_proxy(fx.server.port().unwrap(), 1_500_000).await;
    let mut peer = fx.peer();
    peer.port = proxy_port;
    fx.answer_next_offer(true);

    fx.client.send_files(&peer, &[source]).await.unwrap();

    assert!(cut.load(Ordering::SeqCst), "the connection was never cut");
    assert_eq!(read(&fx, "big.bin"), big);
    assert_eq!(fx.downloaded_names(), vec!["big.bin".to_string()]);
    let events = fx.drain_events();
    for direction in [TransferDirection::Incoming, TransferDirection::Outgoing] {
        let last = events.iter().rev().find_map(|e| match e {
            NodeEvent::TransferProgress { direction: d, bytes_done, bytes_total, .. } if *d == direction => Some((*bytes_done, *bytes_total)),
            _ => None,
        });
        assert_eq!(last, Some((3_000_000, 3_000_000)), "{direction:?}");
    }
    assert!(events.iter().all(|e| !matches!(e, NodeEvent::TransferFailed { .. })));
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn auto_accept_skips_the_prompt_but_still_publishes_events() {
    let mut fx = Fixture::start().await;
    fx.trust.set_auto_accept(fx.client_identity.device_id(), true).unwrap();
    let source = fx.source_file("a.txt", b"hello");

    fx.client.send_files(&fx.peer(), &[source]).await.unwrap();

    assert_eq!(read(&fx, "a.txt"), b"hello");
    let events = fx.drain_events();
    assert!(events.iter().any(|e| matches!(e, NodeEvent::IncomingOffer { .. })));
    assert!(events.iter().any(|e| matches!(e, NodeEvent::OfferClosed { accepted: true, .. })));
    fx.server.stop().await.unwrap();
}
