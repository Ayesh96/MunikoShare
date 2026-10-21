//! Client TLS: verify the peer by certificate fingerprint and present our own certificate.

use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use rustls::ClientConfig;

use super::fingerprint_verifier::{FingerprintVerifier, SeenFingerprint};
use crate::constants::tls::ALPN_HTTP1;
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::net as msg;
use crate::util::tls::ring_provider;

fn tls_error(error: impl std::fmt::Display) -> MunikoError {
    MunikoError::Network(format!("{}: {}", msg::TLS_SETUP_FAILED, error))
}

/// Accepts only a server whose certificate fingerprint equals `peer_id`; presents `identity` if given.
pub fn pinned_client_config(peer_id: &str, identity: Option<&Identity>) -> Result<ClientConfig> {
    build_config(FingerprintVerifier::pinned(peer_id, ring_provider()), identity)
}

/// Accepts any server certificate and reports its fingerprint through the returned handle.
pub fn trust_on_first_use_config(identity: &Identity) -> Result<(ClientConfig, SeenFingerprint)> {
    let seen = SeenFingerprint::default();
    let verifier = FingerprintVerifier::trust_on_first_use(seen.clone(), ring_provider());
    Ok((build_config(verifier, Some(identity))?, seen))
}

fn build_config(verifier: FingerprintVerifier, identity: Option<&Identity>) -> Result<ClientConfig> {
    let builder = ClientConfig::builder_with_provider(ring_provider())
        .with_safe_default_protocol_versions()
        .map_err(tls_error)?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier));
    let mut config = match identity {
        Some(identity) => {
            let cert = CertificateDer::from(identity.cert_der().to_vec());
            let key = PrivatePkcs8KeyDer::from(identity.key_der().to_vec());
            builder.with_client_auth_cert(vec![cert], key.into()).map_err(tls_error)?
        }
        None => builder.with_no_client_auth(),
    };
    config.alpn_protocols = vec![ALPN_HTTP1.to_vec()];
    Ok(config)
}
