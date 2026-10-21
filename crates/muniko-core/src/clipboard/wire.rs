use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::content::ClipboardContent;
use crate::error::{MunikoError, Result};
use crate::messages::clipboard as msg;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WireContent {
    Text { text: String },
    Image { width: u32, height: u32, png_base64: String },
}

/// Body of `POST /api/v1/clipboard`: `{kind, ...content, origin_hash}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardPayload {
    #[serde(flatten)]
    content: WireContent,
    /// SHA-256 of the canonical content, so the receiver can remember it.
    pub origin_hash: Option<String>,
}

impl ClipboardPayload {
    pub fn from_content(content: &ClipboardContent) -> Self {
        let wire = match content {
            ClipboardContent::Text(text) => WireContent::Text { text: text.clone() },
            ClipboardContent::Image { width, height, png } => WireContent::Image { width: *width, height: *height, png_base64: STANDARD.encode(png) },
        };
        Self { content: wire, origin_hash: Some(content.hash()) }
    }

    /// Accepts the legacy `{"text": ...}` body (no `kind`) as text.
    pub fn parse(mut body: Value) -> Result<Self> {
        if let Some(object) = body.as_object_mut() {
            if !object.contains_key("kind") && object.contains_key("text") {
                object.insert("kind".to_string(), json!("text"));
            }
        }
        serde_json::from_value(body).map_err(|e| MunikoError::InvalidState(format!("{}: {e}", msg::BAD_PAYLOAD)))
    }

    pub fn into_content(self) -> Result<ClipboardContent> {
        Ok(match self.content {
            WireContent::Text { text } => ClipboardContent::Text(text),
            WireContent::Image { width, height, png_base64 } => {
                let png = STANDARD.decode(png_base64).map_err(|_| MunikoError::InvalidState(msg::IMAGE_INVALID.to_string()))?;
                ClipboardContent::Image { width, height, png }
            }
        })
    }
}
