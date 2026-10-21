use std::future::Future;
use std::net::SocketAddr;

use reqwest::{Client, RequestBuilder, Response};

use super::addresses::candidate_addresses;
use super::tls::pinned_client_config;
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::net as msg;
use crate::util::errors::error_chain;

/// HTTPS client for one peer: pinned to its device ID, presenting our identity cert.
pub struct PeerClient {
    client: Client,
    base_urls: Vec<String>,
}

impl PeerClient {
    pub fn new(identity: &Identity, peer: &DeviceInfo) -> Result<Self> {
        let config = pinned_client_config(&peer.id, Some(identity))?;
        let client = Client::builder()
            .tls_backend_preconfigured(config)
            .build()
            .map_err(|e| MunikoError::Network(format!("{}: {e}", msg::CLIENT_BUILD_FAILED)))?;
        // SocketAddr formatting brackets IPv6 hosts for us.
        let base_urls = candidate_addresses(&peer.addresses)
            .into_iter()
            .map(|address| format!("https://{}", SocketAddr::new(address, peer.port)))
            .collect();
        Ok(Self { client, base_urls })
    }

    /// Base URLs (`https://ip:port`) in the order they are tried.
    pub fn base_urls(&self) -> &[String] {
        &self.base_urls
    }

    /// Sends the request built for each address in turn; only connect errors move on to the next.
    pub async fn send<F, Fut>(&self, build: F) -> Result<Response>
    where
        F: Fn(Client, String) -> Fut,
        Fut: Future<Output = Result<RequestBuilder>>,
    {
        let mut last_error = None;
        for base_url in &self.base_urls {
            let request = build(self.client.clone(), base_url.clone()).await?;
            match request.send().await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let retry = error.is_connect();
                    last_error = Some(MunikoError::Network(format!("{}: {}", msg::REQUEST_FAILED, error_chain(&error))));
                    if !retry {
                        break;
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| MunikoError::Network(msg::PEER_HAS_NO_ADDRESS.to_string())))
    }
}
