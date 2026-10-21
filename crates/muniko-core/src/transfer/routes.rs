use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{post, put};
use serde::Deserialize;
use axum::{Json, Router};

use super::incoming::{FileTicket, IncomingTransfers, OfferOutcome};
use super::manifest::{Manifest, OfferAccepted};
use super::receiver::FileReceiver;
use super::rejection::Rejection;
use super::responder::OfferResponder;
use crate::constants::transfer::{MAX_OFFER_BODY_BYTES, OFFER_TIMEOUT, ROUTE_TRANSFERS, ROUTE_TRANSFER_FILE, TRANSFER_IDLE_TIMEOUT};
use crate::events::EventBus;
use crate::messages::transfer as msg;
use crate::server::{AuthenticatedPeer, RouteProvider};
use crate::trust::TrustStore;

struct RouteState {
    transfers: Arc<IncomingTransfers>,
    receiver: FileReceiver,
    trust: Arc<dyn TrustStore>,
}

/// Plugs the transfer routes (`POST /api/v1/transfers`, `GET`/`PUT .../files/{index}`) into the shared server.
pub struct TransferRoutes {
    state: Arc<RouteState>,
}

impl TransferRoutes {
    pub fn new(download_dir: PathBuf, events: EventBus, trust: Arc<dyn TrustStore>) -> Self {
        Self::with_timeouts(download_dir, events, trust, OFFER_TIMEOUT, TRANSFER_IDLE_TIMEOUT)
    }

    /// Lets tests shorten how long an offer waits and how long an idle transfer lives.
    pub fn with_timeouts(download_dir: PathBuf, events: EventBus, trust: Arc<dyn TrustStore>, offer_timeout: Duration, idle_timeout: Duration) -> Self {
        let transfers = Arc::new(IncomingTransfers::new(events.clone(), download_dir.clone(), offer_timeout, idle_timeout));
        Self { state: Arc::new(RouteState { transfers, receiver: FileReceiver::new(download_dir, events), trust }) }
    }

    /// Handle the node uses to accept or decline a pending offer.
    pub fn responder(&self) -> OfferResponder {
        OfferResponder::new(self.state.transfers.clone())
    }
}

impl RouteProvider for TransferRoutes {
    fn routes(&self) -> Router {
        Router::new()
            .route(ROUTE_TRANSFERS, post(offer).layer(DefaultBodyLimit::max(MAX_OFFER_BODY_BYTES)))
            .route(ROUTE_TRANSFER_FILE, put(upload).get(received))
            .with_state(self.state.clone())
    }
}

async fn offer(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Json(manifest): Json<Manifest>) -> Response {
    if !state.trust.is_trusted(&peer.device_id) {
        return Rejection::Forbidden(msg::NOT_TRUSTED).into_response();
    }
    let manifest = match manifest.validated() {
        Ok(manifest) => manifest,
        Err(rejection) => return rejection.into_response(),
    };
    match state.transfers.offer(&peer.device_id, manifest, state.trust.auto_accept(&peer.device_id)).await {
        Ok(OfferOutcome::Accepted { transfer_id }) => Json(OfferAccepted { transfer_id }).into_response(),
        Ok(OfferOutcome::Declined) => Rejection::Forbidden(msg::DECLINED).into_response(),
        Err(rejection) => rejection.into_response(),
    }
}

#[derive(Deserialize)]
struct UploadQuery {
    offset: Option<u64>,
}

async fn received(State(state): State<Arc<RouteState>>, peer: AuthenticatedPeer, Path((transfer_id, index)): Path<(String, usize)>) -> Response {
    match state.transfers.received_status(&transfer_id, &peer.device_id, index).await {
        Ok(status) => Json(status).into_response(),
        Err(rejection) => rejection.into_response(),
    }
}

async fn upload(
    State(state): State<Arc<RouteState>>,
    peer: AuthenticatedPeer,
    Path((transfer_id, index)): Path<(String, usize)>,
    Query(query): Query<UploadQuery>,
    body: Body,
) -> Response {
    let ticket = match state.transfers.claim_file(&transfer_id, &peer.device_id, index) {
        Ok(ticket) => ticket,
        Err(rejection) => return rejection.into_response(),
    };
    // Spawned so a dropped connection cannot cancel the handler before it flushes and releases the file.
    let task = tokio::spawn(store_upload(state, ticket, query.offset.unwrap_or(0), body));
    task.await.unwrap_or_else(|_| Rejection::Internal(msg::STORE_FAILED).into_response())
}

async fn store_upload(state: Arc<RouteState>, ticket: FileTicket, offset: u64, body: Body) -> Response {
    match state.receiver.receive(&ticket, offset, body).await {
        Ok(_) => {
            state.transfers.complete_file(&ticket);
            StatusCode::CREATED.into_response()
        }
        Err(rejection) => {
            state.transfers.release_file(&ticket);
            rejection.into_response()
        }
    }
}
