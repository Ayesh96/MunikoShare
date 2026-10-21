//! Shared clipboard: manual push, auto-sync watcher and history, for text and images.

mod access;
mod auto_sender;
mod change_detector;
mod client;
mod content;
mod history;
mod png_codec;
mod recent_hashes;
mod routes;
mod sensitivity;
mod service;
mod shared;
mod validation;
mod watcher;
mod wire;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support;
#[cfg(test)]
mod tests_sync;

pub use access::{ClipboardAccess, SystemClipboard};
pub use client::ClipboardClient;
pub use content::ClipboardContent;
pub use history::{ClipboardHistory, HistoryPreview, HistorySource, HistorySummary};
pub use recent_hashes::RecentHashes;
pub use routes::ClipboardRoutes;
pub use sensitivity::{PlatformProbe, SensitivityProbe};
pub use service::ClipboardService;
pub use shared::ClipboardShared;
pub use validation::ContentProblem;
pub use watcher::ClipboardWatcher;
