use std::sync::Arc;

use reqwest::{Client, RequestBuilder, Response};

use super::profile::{ConfirmBody, PairingProfile};
use crate::constants::pairing::{ROUTE_CONFIRM, ROUTE_REQUEST};
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::pairing as msg;
use crate::net::PeerClient;

/// Calls a peer's pairing endpoints over the pinned, mutually authenticated client.
pub struct PairingClient {
    identity: Arc<Identity>,
}

impl PairingClient {
    pub fn new(identity: Arc<Identity>) -> Self {
        Self { identity }
    }

    /// Asks the peer to show a pairing code.
    pub async fn request(&self, peer: &DeviceInfo, local: &PairingProfile) -> Result<()> {
        let response = self.post(peer, ROUTE_REQUEST, local, msg::REQUEST_SEND_FAILED).await?;
        check_status(response).await.map(|_| ())
    }

    /// Submits the code; on success returns the peer's profile.
    pub async fn confirm(&self, peer: &DeviceInfo, code: &str, local: &PairingProfile) -> Result<PairingProfile> {
        let body = ConfirmBody { code: code.to_string(), profile: local.clone() };
        let response = self.post(peer, ROUTE_CONFIRM, &body, msg::CONFIRM_SEND_FAILED).await?;
        let response = check_status(response).await?;
        response.json().await.map_err(|e| MunikoError::Network(format!("{}: {e}", msg::BAD_RESPONSE)))
    }

    async fn post<B: serde::Serialize>(&self, peer: &DeviceInfo, route: &str, body: &B, context: &str) -> Result<Response> {
        let client = PeerClient::new(&self.identity, peer)?;
        client
            .send(|http, base_url| async move { Ok(build(http, &base_url, route, body)) })
            .await
            .map_err(|e| MunikoError::Network(format!("{context}: {e}")))
    }
}

fn build<B: serde::Serialize>(http: Client, base_url: &str, route: &str, body: &B) -> RequestBuilder {
    http.post(format!("{base_url}{route}")).json(body)
}

async fn check_status(response: Response) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let reason = response.text().await.unwrap_or_default();
    Err(MunikoError::Network(format!("{}: {status} {reason}", msg::REJECTED)))
}
