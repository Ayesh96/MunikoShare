use std::sync::Arc;

use rustls::crypto::{ring, CryptoProvider};

/// The one crypto provider used by every TLS config.
pub fn ring_provider() -> Arc<CryptoProvider> {
    Arc::new(ring::default_provider())
}
