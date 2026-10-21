//! Outgoing connections to peers, shared by every feature.

mod addresses;
mod fingerprint_verifier;
mod peer_client;
mod probe;
mod tls;

pub use addresses::candidate_addresses;
pub use peer_client::PeerClient;
pub use probe::probe;

#[cfg(test)]
pub(crate) use tls::pinned_client_config;
