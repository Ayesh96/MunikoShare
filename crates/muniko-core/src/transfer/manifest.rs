//! The JSON offer a sender posts: which files it wants to send.

use serde::{Deserialize, Serialize};

use super::rejection::Rejection;
use crate::util::sanitize::sanitize_relative_path;
use crate::constants::transfer::{MAX_FILES_PER_OFFER, MAX_OFFER_BYTES, SHA256_HEX_LEN};
use crate::events::FileSummary;
use crate::messages::transfer as msg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    /// Relative, `/`-separated path; a lone file name for a single file.
    #[serde(alias = "name")]
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<ManifestFile>,
}

/// Answer to a resume query: bytes stored so far, and whether the file is already complete.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedStatus {
    pub received: u64,
    pub complete: bool,
}

/// Response to an accepted offer.
#[derive(Debug, Serialize, Deserialize)]
pub struct OfferAccepted {
    pub transfer_id: String,
}

impl Manifest {
    /// Checks the offer and returns it with safe relative paths and lowercase checksums.
    pub fn validated(self) -> Result<Self, Rejection> {
        if self.files.is_empty() {
            return Err(Rejection::BadRequest(msg::EMPTY_OFFER));
        }
        if self.files.len() > MAX_FILES_PER_OFFER {
            return Err(Rejection::BadRequest(msg::TOO_MANY_FILES));
        }
        let files = self.files.into_iter().map(ManifestFile::validated).collect::<Result<Vec<_>, _>>()?;
        let manifest = Self { files };
        if manifest.total_size() > MAX_OFFER_BYTES {
            return Err(Rejection::BadRequest(msg::OFFER_TOO_LARGE));
        }
        Ok(manifest)
    }

    pub fn total_size(&self) -> u64 {
        self.files.iter().fold(0u64, |sum, file| sum.saturating_add(file.size))
    }

    pub fn summaries(&self) -> Vec<FileSummary> {
        self.files.iter().map(|file| FileSummary { name: file.path.clone(), size: file.size }).collect()
    }
}

impl ManifestFile {
    fn validated(self) -> Result<Self, Rejection> {
        let path = sanitize_relative_path(&self.path).ok_or(Rejection::BadRequest(msg::INVALID_PATH))?;
        let sha256 = self.sha256.to_ascii_lowercase();
        if sha256.len() != SHA256_HEX_LEN || !sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Rejection::BadRequest(msg::INVALID_CHECKSUM));
        }
        Ok(Self { path, size: self.size, sha256 })
    }
}
