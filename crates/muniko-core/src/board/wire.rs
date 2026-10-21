//! JSON bodies of the board routes.

use serde::{Deserialize, Serialize};

use super::model::{BoardItem, BoardRecord, DigestEntry};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRequest {
    pub board_id: String,
    pub code: Option<String>,
    /// The joiner's own signed Member item.
    pub member: BoardItem,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinResponse {
    pub board: BoardRecord,
    pub items: Vec<BoardItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DigestResponse {
    pub board_id: String,
    pub entries: Vec<DigestEntry>,
    /// SHA-256 of every board file this device holds; peers learn who can serve what from it.
    #[serde(default)]
    pub have: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemsBody {
    pub items: Vec<BoardItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PushResponse {
    pub added: usize,
    pub rejected: usize,
}
