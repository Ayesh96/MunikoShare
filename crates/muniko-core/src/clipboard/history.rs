use std::sync::{Arc, Mutex};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Serialize;

use super::access::ClipboardAccess;
use super::content::ClipboardContent;
use crate::util::lock::lock;
use crate::constants::clipboard::{HISTORY_CAPACITY, HISTORY_PREVIEW_CHARS, PNG_DATA_URL_PREFIX};
use crate::error::{MunikoError, Result};
use crate::messages::clipboard as msg;
use crate::util::time::unix_now;

/// Where a history item came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HistorySource {
    Local,
    #[serde(rename_all = "camelCase")]
    Device { device_id: String },
}

/// What the UI shows for an item; never carries image bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HistoryPreview {
    Text { preview: String, length: usize },
    Image { width: u32, height: u32, bytes: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySummary {
    pub id: String,
    pub source: HistorySource,
    /// Unix seconds.
    pub timestamp: u64,
    pub pinned: bool,
    pub preview: HistoryPreview,
}

struct Entry {
    summary: HistorySummary,
    hash: String,
    content: ClipboardContent,
}

/// In-memory ring buffer of recent clipboard items, shared by the watcher and the receive route.
/// Local items are only captured while the watcher is polling (some peer has sync enabled).
#[derive(Clone)]
pub struct ClipboardHistory {
    access: Arc<dyn ClipboardAccess>,
    entries: Arc<Mutex<Vec<Entry>>>,
    capacity: usize,
}

impl ClipboardHistory {
    pub fn new(access: Arc<dyn ClipboardAccess>) -> Self {
        Self::with_capacity(access, HISTORY_CAPACITY)
    }

    pub fn with_capacity(access: Arc<dyn ClipboardAccess>, capacity: usize) -> Self {
        Self { access, entries: Arc::default(), capacity: capacity.max(1) }
    }

    /// Adds an item; identical content already present moves to the top and keeps its pin.
    pub fn record(&self, source: HistorySource, content: ClipboardContent) {
        let hash = content.hash();
        let mut entries = lock(&self.entries);
        let pinned = match entries.iter().position(|entry| entry.hash == hash) {
            Some(index) => entries.remove(index).summary.pinned,
            None => false,
        };
        let summary = HistorySummary { id: uuid::Uuid::new_v4().to_string(), source, timestamp: unix_now(), pinned, preview: preview_of(&content) };
        entries.push(Entry { summary, hash, content });
        while entries.len() > self.capacity {
            // Pinned items outlive unpinned ones; if all are pinned the oldest goes.
            let victim = entries.iter().position(|entry| !entry.summary.pinned).unwrap_or(0);
            entries.remove(victim);
        }
    }

    /// Newest first.
    pub fn list(&self) -> Vec<HistorySummary> {
        lock(&self.entries).iter().rev().map(|entry| entry.summary.clone()).collect()
    }

    pub fn text(&self, id: &str) -> Option<String> {
        match self.content(id)? {
            ClipboardContent::Text(text) => Some(text),
            ClipboardContent::Image { .. } => None,
        }
    }

    pub fn image_png(&self, id: &str) -> Option<Vec<u8>> {
        match self.content(id)? {
            ClipboardContent::Image { png, .. } => Some(png),
            ClipboardContent::Text(_) => None,
        }
    }

    /// The image as a `data:image/png;base64,...` URL for direct use in an `<img>`.
    pub fn image_data_url(&self, id: &str) -> Option<String> {
        self.image_png(id).map(|png| format!("{PNG_DATA_URL_PREFIX}{}", STANDARD.encode(png)))
    }

    /// Writes the item to the system clipboard. This is a deliberate local copy,
    /// so the watcher treats it as a normal change and syncs it to peers.
    pub async fn copy_back(&self, id: &str) -> Result<()> {
        let content = self.content(id).ok_or_else(|| MunikoError::InvalidState(msg::HISTORY_ITEM_MISSING.to_string()))?;
        let access = self.access.clone();
        tokio::task::spawn_blocking(move || access.write_content(&content))
            .await
            .map_err(|e| MunikoError::InvalidState(format!("{}: {e}", msg::TASK_FAILED)))?
    }

    /// Returns false when the id is unknown.
    pub fn set_pinned(&self, id: &str, pinned: bool) -> bool {
        match lock(&self.entries).iter_mut().find(|entry| entry.summary.id == id) {
            Some(entry) => {
                entry.summary.pinned = pinned;
                true
            }
            None => false,
        }
    }

    pub fn clear(&self) {
        lock(&self.entries).clear();
    }

    fn content(&self, id: &str) -> Option<ClipboardContent> {
        lock(&self.entries).iter().find(|entry| entry.summary.id == id).map(|entry| entry.content.clone())
    }
}

fn preview_of(content: &ClipboardContent) -> HistoryPreview {
    match content {
        ClipboardContent::Text(text) => HistoryPreview::Text { preview: text.chars().take(HISTORY_PREVIEW_CHARS).collect(), length: text.chars().count() },
        ClipboardContent::Image { width, height, png } => HistoryPreview::Image { width: *width, height: *height, bytes: png.len() },
    }
}
