use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

use super::verifier::AnyClientCertVerifier;
use crate::constants::tls::{ALPN_H2, ALPN_HTTP1};
use crate::error::{MunikoError, Result};
use crate::identity::Identity;
use crate::messages::server as msg;
use crate::util::tls::ring_provider;

fn tls_error(error: impl std::fmt::Display) -> MunikoError {
    MunikoError::Server(format!("{}: {}", msg::TLS_SETUP_FAILED, error))
}

/// Server config presenting the device cert and optionally asking clients for theirs.
pub fn server_config(identity: &Identity) -> Result<rustls::ServerConfig> {
    let provider = ring_provider();
    let cert = CertificateDer::from(identity.cert_der().to_vec());
    let key = PrivatePkcs8KeyDer::from(identity.key_der().to_vec());
    let mut config = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(tls_error)?
        .with_client_cert_verifier(Arc::new(AnyClientCertVerifier::new(provider)))
        .with_single_cert(vec![cert], key.into())
        .map_err(tls_error)?;
    config.alpn_protocols = vec![ALPN_H2.to_vec(), ALPN_HTTP1.to_vec()];
    Ok(config)
}
