//! Salted, iterated hashing of a protected board's join code.

use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::model::JoinSecret;
use crate::constants::board::{JOIN_HASH_ROUNDS, JOIN_SALT_BYTES};

impl JoinSecret {
    pub fn create(code: &str) -> Self {
        let mut salt = [0u8; JOIN_SALT_BYTES];
        // The system RNG failing is unrecoverable and would only weaken the salt.
        SystemRandom::new().fill(&mut salt).expect("system random source");
        Self { hash: hex::encode(derive(&salt, code)), salt: hex::encode(salt) }
    }

    pub fn verify(&self, code: &str) -> bool {
        let (Ok(salt), Ok(expected)) = (hex::decode(&self.salt), hex::decode(&self.hash)) else {
            return false;
        };
        derive(&salt, code).ct_eq(&expected).into()
    }
}

fn derive(salt: &[u8], code: &str) -> Vec<u8> {
    let mut digest = Sha256::new().chain_update(salt).chain_update(code.as_bytes()).finalize().to_vec();
    for _ in 0..JOIN_HASH_ROUNDS {
        digest = Sha256::new().chain_update(salt).chain_update(&digest).finalize().to_vec();
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_right_code() {
        let secret = JoinSecret::create("1234abcd");
        assert!(secret.verify("1234abcd"));
        assert!(!secret.verify("1234abce"));
    }
}
