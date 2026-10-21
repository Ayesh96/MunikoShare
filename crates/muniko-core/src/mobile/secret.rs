//! Random secrets and their digests.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};

use crate::constants::mobile::TOKEN_BYTES;
use crate::error::{MunikoError, Result};
use crate::messages::mobile as msg;

/// A CSPRNG value as base64url text, usable in URLs and cookies.
pub fn random_secret() -> Result<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    SystemRandom::new().fill(&mut bytes).map_err(|_| MunikoError::Server(msg::RANDOM_FAILED.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// Digest used as the lookup key so the map never compares raw tokens.
pub fn digest(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}
