use serde::{Deserialize, Serialize};

use crate::constants::history::CLIPBOARD_PREVIEW_CHARS;
use crate::util::time::unix_now;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HistoryKind {
    FileReceived,
    FileSent,
    TransferFailed,
    ClipboardReceived,
    ClipboardSent,
    Paired,
    Unpaired,
}

impl HistoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileReceived => "fileReceived",
            Self::FileSent => "fileSent",
            Self::TransferFailed => "transferFailed",
            Self::ClipboardReceived => "clipboardReceived",
            Self::ClipboardSent => "clipboardSent",
            Self::Paired => "paired",
            Self::Unpaired => "unpaired",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        [
            Self::FileReceived,
            Self::FileSent,
            Self::TransferFailed,
            Self::ClipboardReceived,
            Self::ClipboardSent,
            Self::Paired,
            Self::Unpaired,
        ]
        .into_iter()
        .find(|kind| kind.as_str() == value)
    }
}

/// A stored history row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: u64,
    pub kind: HistoryKind,
    pub device_id: String,
    pub detail: String,
}

/// An entry not yet stored; the store assigns the id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewHistoryEntry {
    pub timestamp: u64,
    pub kind: HistoryKind,
    pub device_id: String,
    pub detail: String,
}

impl NewHistoryEntry {
    pub fn new(kind: HistoryKind, device_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { timestamp: unix_now(), kind, device_id: device_id.into(), detail: detail.into() }
    }
}

/// First characters of clipboard text, cut on a char boundary.
pub fn clipboard_preview(text: &str) -> String {
    text.chars().take(CLIPBOARD_PREVIEW_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_truncates_on_char_boundary() {
        let text = "é".repeat(CLIPBOARD_PREVIEW_CHARS + 10);
        let preview = clipboard_preview(&text);
        assert_eq!(preview.chars().count(), CLIPBOARD_PREVIEW_CHARS);
        assert_eq!(clipboard_preview("short"), "short");
    }

    #[test]
    fn kind_string_round_trips() {
        assert_eq!(HistoryKind::parse(HistoryKind::ClipboardSent.as_str()), Some(HistoryKind::ClipboardSent));
        assert_eq!(HistoryKind::parse("nope"), None);
    }

    #[test]
    fn kind_serializes_camel_case() {
        assert_eq!(serde_json::to_string(&HistoryKind::FileReceived).unwrap(), "\"fileReceived\"");
    }
}
