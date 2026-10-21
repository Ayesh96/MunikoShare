//! Pairing: exchange a 6-digit code to trust another device.

mod client;
mod code;
mod profile;
mod routes;
mod service;

#[cfg(test)]
mod tests;

pub use client::PairingClient;
pub use code::{generate_code, PendingBook, Verdict};
pub use profile::PairingProfile;
pub use routes::PairingRoutes;
pub use service::PairingService;
