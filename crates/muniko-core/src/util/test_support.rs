//! Fixtures shared by the crate's unit tests.

use std::sync::Arc;

use crate::identity::Identity;
use crate::util::fingerprint::fingerprint;

/// A fresh self-signed identity for the local test server.
pub fn test_identity() -> Identity {
    let certified = rcgen::generate_simple_self_signed(vec!["muniko.local".to_string()]).unwrap();
    let cert_der = certified.cert.der().to_vec();
    let key_der = certified.signing_key.serialize_der();
    Identity::new(fingerprint(&cert_der), cert_der, key_der)
}

/// `test_identity` wrapped in an `Arc`, for services that share it.
pub fn shared_test_identity() -> Arc<Identity> {
    Arc::new(test_identity())
}
