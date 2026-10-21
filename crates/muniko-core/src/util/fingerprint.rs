use sha2::{Digest, Sha256};

/// Hex SHA-256 of a DER certificate; this is the device ID.
pub fn fingerprint(cert_der: &[u8]) -> String {
    hex::encode(Sha256::digest(cert_der))
}
