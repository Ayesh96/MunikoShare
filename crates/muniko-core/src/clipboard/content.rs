use sha2::{Digest, Sha256};

use super::png_codec;
use super::validation::ContentProblem;
use crate::constants::clipboard::{HASH_IMAGE_TAG, HASH_TEXT_TAG, MAX_CLIPBOARD_BYTES, MAX_IMAGE_PNG_BYTES};

/// What the clipboard holds: text, or a PNG-encoded image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardContent {
    Text(String),
    Image { width: u32, height: u32, png: Vec<u8> },
}

impl ClipboardContent {
    /// Lowercase hex SHA-256 of the canonical content; identical on every device.
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        match self {
            Self::Text(text) => {
                hasher.update(HASH_TEXT_TAG);
                hasher.update(text.as_bytes());
            }
            Self::Image { width, height, png } => {
                hasher.update(HASH_IMAGE_TAG);
                hasher.update(width.to_be_bytes());
                hasher.update(height.to_be_bytes());
                hasher.update(png);
            }
        }
        hex::encode(hasher.finalize())
    }

    /// Why this content must not be pushed or auto-synced, if it must not.
    pub fn problem(&self) -> Option<ContentProblem> {
        match self {
            Self::Text(text) if text.is_empty() => Some(ContentProblem::EmptyText),
            Self::Text(text) if text.len() > MAX_CLIPBOARD_BYTES => Some(ContentProblem::TextTooLarge),
            Self::Text(_) => None,
            Self::Image { png, .. } if png.is_empty() => Some(ContentProblem::EmptyImage),
            Self::Image { png, .. } if png.len() > MAX_IMAGE_PNG_BYTES => Some(ContentProblem::ImageTooLarge),
            Self::Image { width, height, png } => match png_codec::dimensions(png) {
                Ok(found) if found == (*width, *height) => None,
                _ => Some(ContentProblem::InvalidImage),
            },
        }
    }
}
