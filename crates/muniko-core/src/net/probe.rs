use std::net::{IpAddr, SocketAddr};

use reqwest::Client;

use super::tls::trust_on_first_use_config;
use crate::constants::info::{PROBE_TIMEOUT, ROUTE_INFO};
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::net as msg;
use crate::util::errors::error_chain;

fn network_error(context: &str, detail: impl std::fmt::Display) -> MunikoError {
    MunikoError::Network(format!("{context}: {detail}"))
}

/// Asks the device at `ip:port` who it is, without knowing its ID beforehand (trust on first use).
pub async fn probe(identity: &Identity, ip: IpAddr, port: u16) -> Result<DeviceInfo> {
    let (config, seen) = trust_on_first_use_config(identity)?;
    let client = Client::builder()
        .tls_backend_preconfigured(config)
        .timeout(PROBE_TIMEOUT)
        .build()
        .map_err(|e| network_error(msg::CLIENT_BUILD_FAILED, e))?;
    // SocketAddr formatting brackets IPv6 hosts for us.
    let url = format!("https://{}{ROUTE_INFO}", SocketAddr::new(ip, port));
    let response = client.get(url).send().await.map_err(|e| network_error(msg::PROBE_FAILED, error_chain(&e)))?;
    let status = response.status();
    if !status.is_success() {
        return Err(network_error(msg::PROBE_REJECTED, status));
    }
    let info: DeviceInfo = response.json().await.map_err(|e| network_error(msg::PROBE_BAD_RESPONSE, e))?;
    let certificate_id = seen.get().ok_or_else(|| MunikoError::Network(msg::PROBE_NO_CERT.to_string()))?;
    // The claimed ID must be the one the TLS handshake proved, or anyone could impersonate a device.
    if info.id != certificate_id {
        return Err(MunikoError::Network(msg::PROBE_ID_MISMATCH.to_string()));
    }
    Ok(DeviceInfo { addresses: vec![ip], port, ..info })
}
