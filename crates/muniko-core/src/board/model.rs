use serde::{Deserialize, Serialize};

use super::canonical::CanonicalWriter;
use super::hex_serde;
use crate::constants::board::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BoardMode {
    Open,
    Protected,
}

impl BoardMode {
    pub fn as_str(self) -> &'static str {
        match self {
            BoardMode::Open => MODE_OPEN,
            BoardMode::Protected => MODE_PROTECTED,
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            MODE_OPEN => Some(BoardMode::Open),
            MODE_PROTECTED => Some(BoardMode::Protected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub mode: BoardMode,
    pub created_at: u64,
}

/// Salted hash of a protected board's join code; replicated to members so any can check joins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinSecret {
    pub salt: String,
    pub hash: String,
}

/// A board plus its join secret; the secret never reaches the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardRecord {
    #[serde(flatten)]
    pub board: Board,
    pub join_secret: Option<JoinSecret>,
}

/// What `GET /api/v1/board` tells any authenticated device about a board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardInfo {
    #[serde(flatten)]
    pub board: Board,
    pub member_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ItemKind {
    Text { text: String },
    File { name: String, size: u64, sha256: String },
    Tombstone { target_id: String },
    Member {
        device_id: String,
        name: String,
        #[serde(with = "hex_serde")]
        cert_der: Vec<u8>,
    },
}

impl ItemKind {
    pub fn tag(&self) -> &'static str {
        match self {
            ItemKind::Text { .. } => KIND_TEXT,
            ItemKind::File { .. } => KIND_FILE,
            ItemKind::Tombstone { .. } => KIND_TOMBSTONE,
            ItemKind::Member { .. } => KIND_MEMBER,
        }
    }
}

/// One signed entry of the append-only board log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardItem {
    pub id: String,
    pub board_id: String,
    pub author_id: String,
    pub created_at: u64,
    pub kind: ItemKind,
    #[serde(with = "hex_serde")]
    pub signature: Vec<u8>,
}

impl BoardItem {
    /// The exact bytes that are signed; the signature itself is excluded.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut writer = CanonicalWriter::new();
        writer.put_str(CANONICAL_DOMAIN);
        writer.put_str(&self.id);
        writer.put_str(&self.board_id);
        writer.put_str(&self.author_id);
        writer.put_u64(self.created_at);
        writer.put_str(self.kind.tag());
        match &self.kind {
            ItemKind::Text { text } => writer.put_str(text),
            ItemKind::File { name, size, sha256 } => {
                writer.put_str(name);
                writer.put_u64(*size);
                writer.put_str(sha256);
            }
            ItemKind::Tombstone { target_id } => writer.put_str(target_id),
            ItemKind::Member { device_id, name, cert_der } => {
                writer.put_str(device_id);
                writer.put_str(name);
                writer.put_bytes(cert_der);
            }
        }
        writer.finish()
    }

    /// Hex SHA-256 over the signed content and signature; peers compare these in digests.
    pub fn hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.canonical_bytes());
        hasher.update(&self.signature);
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DigestEntry {
    pub id: String,
    pub hash: String,
    pub created_at: u64,
}
