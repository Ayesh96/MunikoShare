use std::sync::Arc;

use reqwest::{header, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::model::{BoardInfo, BoardItem};
use super::wire::{DigestResponse, FetchRequest, ItemsBody, JoinRequest, JoinResponse, PushResponse};
use crate::constants::board::*;
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::board as msg;
use crate::net::PeerClient;

/// Calls the board routes of one peer at a time over the shared pinned client.
pub struct BoardClient {
    identity: Arc<Identity>,
}

impl BoardClient {
    pub fn new(identity: Arc<Identity>) -> Self {
        Self { identity }
    }

    /// `None` when the peer has no board.
    pub async fn info(&self, peer: &DeviceInfo) -> Result<Option<BoardInfo>> {
        self.call::<(), BoardInfo>(peer, Method::GET, ROUTE_BOARD, None).await
    }

    pub async fn join(&self, peer: &DeviceInfo, request: &JoinRequest) -> Result<JoinResponse> {
        self.expect(peer, Method::POST, ROUTE_BOARD_JOIN, Some(request)).await
    }

    pub async fn digest(&self, peer: &DeviceInfo) -> Result<DigestResponse> {
        self.expect::<(), _>(peer, Method::GET, ROUTE_BOARD_DIGEST, None).await
    }

    /// Opens a peer's cached file from byte `offset`; the flag says whether the peer honoured the range.
    pub async fn open_file(&self, peer: &DeviceInfo, sha256: &str, offset: u64) -> Result<(Response, bool)> {
        let client = PeerClient::new(&self.identity, peer)?;
        let path = format!("{ROUTE_BOARD_FILES}/{sha256}");
        let response = client
            .send(|http, base_url| {
                let mut request = http.get(format!("{base_url}{path}"));
                if offset > 0 {
                    request = request.header(header::RANGE, format!("{RANGE_UNIT_PREFIX}{offset}-"));
                }
                async move { Ok(request) }
            })
            .await
            .map_err(|e| MunikoError::Network(format!("{}: {e}", msg::REQUEST_FAILED)))?;
        match response.status() {
            StatusCode::OK => Ok((response, false)),
            StatusCode::PARTIAL_CONTENT => Ok((response, true)),
            status => Err(MunikoError::Network(format!("{}: {status}", msg::PEER_REJECTED))),
        }
    }

    pub async fn fetch(&self, peer: &DeviceInfo, ids: Vec<String>) -> Result<Vec<BoardItem>> {
        let body = FetchRequest { ids };
        Ok(self.expect::<_, ItemsBody>(peer, Method::POST, ROUTE_BOARD_FETCH, Some(&body)).await?.items)
    }

    pub async fn push(&self, peer: &DeviceInfo, items: Vec<BoardItem>) -> Result<PushResponse> {
        self.expect(peer, Method::POST, ROUTE_BOARD_ITEMS, Some(&ItemsBody { items })).await
    }

    async fn expect<B: Serialize, R: DeserializeOwned>(&self, peer: &DeviceInfo, method: Method, path: &str, body: Option<&B>) -> Result<R> {
        self.call(peer, method, path, body).await?.ok_or_else(|| MunikoError::Network(msg::NO_BOARD.to_string()))
    }

    async fn call<B: Serialize, R: DeserializeOwned>(&self, peer: &DeviceInfo, method: Method, path: &str, body: Option<&B>) -> Result<Option<R>> {
        let client = PeerClient::new(&self.identity, peer)?;
        let response = client
            .send(|http, base_url| {
                let mut request = http.request(method.clone(), format!("{base_url}{path}"));
                if let Some(body) = body {
                    request = request.json(body);
                }
                async move { Ok(request) }
            })
            .await
            .map_err(|e| MunikoError::Network(format!("{}: {e}", msg::REQUEST_FAILED)))?;
        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            let reason = response.text().await.unwrap_or_default();
            return Err(MunikoError::Network(format!("{}: {status} {reason}", msg::PEER_REJECTED)));
        }
        let parsed = response.json::<R>().await.map_err(|e| MunikoError::Network(format!("{}: {e}", msg::BAD_REPLY)))?;
        Ok(Some(parsed))
    }
}
