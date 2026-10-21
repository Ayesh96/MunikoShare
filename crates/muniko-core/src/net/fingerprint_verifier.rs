use std::sync::{Arc, Mutex};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};

use crate::messages::net as msg;
use crate::util::fingerprint::fingerprint;

/// Shared slot where the verifier leaves the server certificate fingerprint it saw.
#[derive(Debug, Clone, Default)]
pub struct SeenFingerprint(Arc<Mutex<Option<String>>>);

impl SeenFingerprint {
    pub fn get(&self) -> Option<String> {
        self.0.lock().ok().and_then(|seen| seen.clone())
    }

    fn record(&self, value: String) {
        if let Ok(mut seen) = self.0.lock() {
            *seen = Some(value);
        }
    }
}

/// Verifies a server by certificate fingerprint: pinned to an expected ID, or trust on first use.
#[derive(Debug)]
pub struct FingerprintVerifier {
    expected_id: Option<String>,
    seen: SeenFingerprint,
    provider: Arc<CryptoProvider>,
}

impl FingerprintVerifier {
    pub fn pinned(peer_id: &str, provider: Arc<CryptoProvider>) -> Self {
        Self { expected_id: Some(peer_id.to_string()), seen: SeenFingerprint::default(), provider }
    }

    /// Accepts any certificate and records its fingerprint into `seen`.
    pub fn trust_on_first_use(seen: SeenFingerprint, provider: Arc<CryptoProvider>) -> Self {
        Self { expected_id: None, seen, provider }
    }
}

impl ServerCertVerifier for FingerprintVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        let actual = fingerprint(end_entity.as_ref());
        let matches = self.expected_id.as_ref().is_none_or(|expected| *expected == actual);
        self.seen.record(actual);
        if matches {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(msg::CERT_MISMATCH.to_string()))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}
