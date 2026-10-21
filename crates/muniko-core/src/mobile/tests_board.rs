//! The phone's board API over a real loopback server and a real in-memory board.

use reqwest::{header, StatusCode};
use serde_json::{json, Value};

use super::tests::{read_until, Fixture};
use crate::board::{BoardMode, ViewKind};
use crate::constants::mobile::*;
use crate::lifecycle::Lifecycle;

const FILE_BYTES: &[u8] = b"0123456789abcdef";

fn with_board(fx: &Fixture) {
    fx.board.create_board("Office", BoardMode::Open, None).unwrap();
}

#[tokio::test]
async fn board_endpoints_need_a_session() {
    let mut fx = Fixture::start().await;
    let requests = [
        fx.client.get(fx.url(ROUTE_BOARD)),
        fx.client.post(fx.url(ROUTE_BOARD_TEXT)).body(r#"{"text":"x"}"#),
        fx.client.post(fx.url(ROUTE_BOARD_FILES)).header(FILE_NAME_HEADER, "a.txt").body("x"),
        fx.client.get(fx.url("/api/m/board/files/abc")),
    ];
    for request in requests {
        assert_eq!(request.send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    }
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn without_a_board_reads_are_empty_and_posts_conflict() {
    let mut fx = Fixture::start().await;
    let phone = fx.login().await;
    let body: Value = fx.get(&phone, ROUTE_BOARD).send().await.unwrap().json().await.unwrap();
    assert_eq!(body, json!({ "board": null, "items": [] }));
    assert_eq!(fx.post(&phone, ROUTE_BOARD_TEXT).body(r#"{"text":"hi"}"#).send().await.unwrap().status(), StatusCode::CONFLICT);
    let file = fx.post(&phone, ROUTE_BOARD_FILES).header(FILE_NAME_HEADER, "a.txt").body("x").send().await.unwrap();
    assert_eq!(file.status(), StatusCode::CONFLICT);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn posted_text_is_listed_and_lands_on_the_board() {
    let mut fx = Fixture::start().await;
    with_board(&fx);
    let phone = fx.login().await;
    let created = fx.post(&phone, ROUTE_BOARD_TEXT).body(r#"{"text":"from phone"}"#).send().await.unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let view: Value = created.json().await.unwrap();
    assert_eq!(view["kind"], json!({ "type": "text", "text": "from phone" }));
    assert_eq!(view["mine"], true);
    assert!(fx.board.items().unwrap().iter().any(|item| matches!(&item.kind, ViewKind::Text { text } if text == "from phone")));

    let list: Value = fx.get(&phone, ROUTE_BOARD).send().await.unwrap().json().await.unwrap();
    assert_eq!(list["board"]["name"], "Office");
    assert_eq!(list["board"]["mode"], "open");
    assert_eq!(list["items"][0]["id"], view["id"]);

    assert_eq!(fx.post(&phone, ROUTE_BOARD_TEXT).body(r#"{"text":"  "}"#).send().await.unwrap().status(), StatusCode::BAD_REQUEST);
    assert_eq!(fx.post(&phone, ROUTE_BOARD_TEXT).body("nope").send().await.unwrap().status(), StatusCode::BAD_REQUEST);
    let huge = json!({ "text": "x".repeat(100 * 1024) }).to_string();
    assert_eq!(fx.post(&phone, ROUTE_BOARD_TEXT).body(huge).send().await.unwrap().status(), StatusCode::PAYLOAD_TOO_LARGE);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn posted_file_downloads_back_with_range() {
    let mut fx = Fixture::start().await;
    with_board(&fx);
    let phone = fx.login().await;
    let created = fx.post(&phone, ROUTE_BOARD_FILES).header(FILE_NAME_HEADER, "caf%C3%A9%20notes.txt").body(FILE_BYTES).send().await.unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let view: Value = created.json().await.unwrap();
    assert_eq!(view["kind"]["name"], "café notes.txt");
    assert_eq!(view["kind"]["size"], FILE_BYTES.len());
    let url = format!("/api/m/board/files/{}", view["id"].as_str().unwrap());

    let whole = fx.get(&phone, &url).send().await.unwrap();
    assert_eq!(whole.status(), StatusCode::OK);
    assert!(whole.headers()[header::CONTENT_DISPOSITION].to_str().unwrap().contains("filename*=UTF-8''caf%C3%A9%20notes%2Etxt"));
    assert_eq!(whole.bytes().await.unwrap().as_ref(), FILE_BYTES);

    let part = fx.get(&phone, &url).header(header::RANGE, "bytes=4-7").send().await.unwrap();
    assert_eq!(part.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(part.headers()[header::CONTENT_RANGE], "bytes 4-7/16");
    assert_eq!(part.bytes().await.unwrap().as_ref(), b"4567");

    assert_eq!(fx.get(&phone, "/api/m/board/files/missing").send().await.unwrap().status(), StatusCode::NOT_FOUND);
    let text: Value = fx.post(&phone, ROUTE_BOARD_TEXT).body(r#"{"text":"t"}"#).send().await.unwrap().json().await.unwrap();
    let not_file = format!("/api/m/board/files/{}", text["id"].as_str().unwrap());
    assert_eq!(fx.get(&phone, &not_file).send().await.unwrap().status(), StatusCode::NOT_FOUND);
    fx.server.stop().await.unwrap();
}

#[tokio::test]
async fn board_post_sends_one_board_event_to_the_phone() {
    let mut fx = Fixture::start().await;
    with_board(&fx);
    let phone = fx.login().await;
    let mut stream = fx.get(&phone, ROUTE_EVENTS).send().await.unwrap();
    assert_eq!(stream.status(), StatusCode::OK);
    fx.post(&phone, ROUTE_BOARD_TEXT).body(r#"{"text":"ping"}"#).send().await.unwrap();
    assert!(read_until(&mut stream, "event: board").await.contains("event: board"));
    fx.server.stop().await.unwrap();
}
