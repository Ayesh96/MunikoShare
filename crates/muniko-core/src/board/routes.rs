use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::{header, HeaderMap};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use super::files::{serve_file, FileCache};
use super::guard::JoinGuard;
use super::members::self_certified;
use super::model::{BoardMode, BoardRecord};
use super::outbox::Outbox;
use super::replica::Replica;
use super::wire::*;
use crate::constants::board::*;
use crate::constants::server::MAX_SMALL_JSON_BYTES;
use crate::error::MunikoError;
use crate::events::{EventBus, NodeEvent};
use crate::messages::board as msg;
use crate::server::{AuthenticatedPeer, RouteProvider};
use crate::util::time::unix_now;

struct RouteState {
    replica: Arc<Replica>,
    guard: JoinGuard,
    events: EventBus,
    outbox: Arc<Outbox>,
    cache: Arc<FileCache>,
}

/// Plugs the board routes into the shared server; all need a client certificate.
pub struct BoardRoutes {
    state: Arc<RouteState>,
}

impl BoardRoutes {
    pub fn new(replica: Arc<Replica>, events: EventBus, outbox: Arc<Outbox>, cache: Arc<FileCache>) -> Self {
        Self { state: Arc::new(RouteState { replica, guard: JoinGuard::default(), events, outbox, cache }) }
    }
}

impl RouteProvider for BoardRoutes {
    fn routes(&self) -> Router {
        Router::new()
            .route(ROUTE_BOARD, get(info))
            .route(ROUTE_BOARD_JOIN, post(join).layer(small_json()))
            .route(ROUTE_BOARD_DIGEST, get(digest))
            .route(ROUTE_BOARD_FETCH, post(fetch).layer(small_json()))
            .route(ROUTE_BOARD_ITEMS, post(push).layer(DefaultBodyLimit::max(MAX_PUSH_BODY_BYTES)))
            .route(ROUTE_BOARD_FILE, get(file))
            .with_state(self.state.clone())
    }
}

fn small_json() -> DefaultBodyLimit {
    DefaultBodyLimit::max(MAX_SMALL_JSON_BYTES)
}

fn failure(status: StatusCode, text: &'static str) -> Response {
    (status, text).into_response()
}

fn internal(error: MunikoError) -> Response {
    let status = match &error {
        MunikoError::Board(board) => board.status(),
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string()).into_response()
}

/// The local board, provided the caller is one of its members.
fn member_board(state: &RouteState, peer: &AuthenticatedPeer) -> Result<BoardRecord, Response> {
    let record = state.replica.record().map_err(internal)?.ok_or_else(|| failure(StatusCode::NOT_FOUND, msg::NO_BOARD))?;
    let members = state.replica.members().map_err(internal)?;
    if members.contains(&peer.device_id) {
        Ok(record)
    } else {
        Err(failure(StatusCode::FORBIDDEN, msg::NOT_A_MEMBER))
    }
}

fn changed(state: &RouteState, board_id: &str) {
    state.events.publish(NodeEvent::BoardChanged { board_id: board_id.to_string() });
}

async fn info(State(state): State<Arc<RouteState>>, _peer: AuthenticatedPeer) -> Response {
    match state.replica.info() {
        Ok(Some(info)) => Json(info).into_response(),
        Ok(None) => failure(StatusCode::NOT_FOUND, msg::NO_BOARD),
        Err(error) => internal(error),
    }
}

async fn join(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Json(request): Json<JoinRequest>) -> Response {
    let record = match state.replica.record() {
        Ok(Some(record)) => record,
        Ok(None) => return failure(StatusCode::NOT_FOUND, msg::NO_BOARD),
        Err(error) => return internal(error),
    };
    if record.board.id != request.board_id {
        return failure(StatusCode::CONFLICT, msg::BOARD_MISMATCH);
    }
    if !state.guard.allows(&peer.device_id) {
        return failure(StatusCode::TOO_MANY_REQUESTS, msg::TOO_MANY_ATTEMPTS);
    }
    if !code_accepted(&record, request.code.as_deref()) {
        state.guard.record_failure(&peer.device_id);
        return failure(StatusCode::FORBIDDEN, msg::WRONG_CODE);
    }
    state.guard.clear(&peer.device_id);
    if request.member.author_id != peer.device_id || self_certified(&request.member).is_none() {
        return failure(StatusCode::BAD_REQUEST, msg::INVALID_MEMBER);
    }
    admit_member(&state, record, request).await
}

fn code_accepted(record: &BoardRecord, code: Option<&str>) -> bool {
    match (record.board.mode, &record.join_secret) {
        (BoardMode::Open, _) => true,
        (BoardMode::Protected, Some(secret)) => code.is_some_and(|code| secret.verify(code)),
        (BoardMode::Protected, None) => false,
    }
}

async fn admit_member(state: &RouteState, record: BoardRecord, request: JoinRequest) -> Response {
    let accepted = match state.replica.accept(vec![request.member.clone()], unix_now()) {
        Ok(accepted) => accepted,
        Err(error) => return internal(error),
    };
    if accepted.rejected > 0 {
        return failure(StatusCode::BAD_REQUEST, msg::INVALID_MEMBER);
    }
    if !accepted.added.is_empty() {
        changed(state, &record.board.id);
        // Other members only learn the newcomer through us; they would refuse its pushes until then.
        accepted.added.into_iter().for_each(|item| state.outbox.push(item));
    }
    match state.replica.store().all() {
        Ok(items) => Json(JoinResponse { board: record, items }).into_response(),
        Err(error) => internal(error),
    }
}

async fn digest(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer) -> Response {
    let record = match member_board(&state, &peer) {
        Ok(record) => record,
        Err(response) => return response,
    };
    match state.replica.store().digest() {
        Ok(entries) => Json(DigestResponse { board_id: record.board.id, entries, have: state.cache.held() }).into_response(),
        Err(error) => internal(error),
    }
}

async fn fetch(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Json(request): Json<FetchRequest>) -> Response {
    if let Err(response) = member_board(&state, &peer) {
        return response;
    }
    if request.ids.len() > MAX_FETCH_IDS {
        return failure(StatusCode::PAYLOAD_TOO_LARGE, msg::TOO_MANY_ITEMS);
    }
    let mut items = Vec::new();
    for id in &request.ids {
        match state.replica.store().get(id) {
            Ok(found) => items.extend(found),
            Err(error) => return internal(error),
        }
    }
    Json(ItemsBody { items }).into_response()
}

async fn push(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Json(body): Json<ItemsBody>) -> Response {
    let record = match member_board(&state, &peer) {
        Ok(record) => record,
        Err(response) => return response,
    };
    if body.items.len() > MAX_PUSH_ITEMS {
        return failure(StatusCode::PAYLOAD_TOO_LARGE, msg::TOO_MANY_ITEMS);
    }
    match state.replica.accept(body.items, unix_now()) {
        Ok(accepted) => {
            if !accepted.added.is_empty() {
                changed(&state, &record.board.id);
            }
            Json(PushResponse { added: accepted.added.len(), rejected: accepted.rejected }).into_response()
        }
        Err(error) => internal(error),
    }
}

async fn file(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Path(sha256): Path<String>, headers: HeaderMap) -> Response {
    if let Err(response) = member_board(&state, &peer) {
        return response;
    }
    let range = headers.get(header::RANGE).and_then(|value| value.to_str().ok());
    serve_file(&state.cache, &sha256, range).await
}
