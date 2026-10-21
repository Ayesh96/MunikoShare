//! What a phone can do once it has a session.

use std::path::PathBuf;

use tokio::sync::broadcast;

use crate::board::BoardService;
use super::event::MobileEvent;
use super::inbox::{InboxItem, InboxView};
use super::service::MobileService;
use super::session::SharedText;
use crate::constants::mobile::PHONE_ID_PREFIX;
use crate::error::{MunikoError, Result};
use crate::events::NodeEvent;
use crate::messages::mobile as msg;
use crate::util::lock::lock;
use crate::util::time::unix_now;

impl MobileService {
    pub(super) fn download_dir(&self) -> PathBuf {
        self.inner.download_dir.clone()
    }

    pub(super) fn session_ttl_secs(&self) -> u64 {
        self.inner.limits.session_ttl_secs
    }

    pub(super) fn max_upload_bytes(&self) -> u64 {
        self.inner.limits.max_upload_bytes
    }

    pub(super) fn logout(&self, session_id: &str) {
        self.revoke(session_id);
    }

    pub(super) fn record_upload(&self, session_id: &str, path: PathBuf) {
        tracing::info!(path = %path.display(), "{}", msg::UPLOAD_RECEIVED);
        self.inner.events.publish(NodeEvent::FileReceived { from_device_id: phone_id(session_id), path });
    }

    /// Puts the text on the desktop clipboard (a blocking call, so off the async threads).
    pub(super) async fn accept_text(&self, session_id: &str, text: String) -> Result<()> {
        let clipboard = self.inner.clipboard.clone();
        let content = text.clone();
        tokio::task::spawn_blocking(move || clipboard.write_text(&content))
            .await
            .map_err(|e| MunikoError::Clipboard(format!("{}: {e}", msg::CLIPBOARD_WRITE_FAILED)))??;
        tracing::info!("{}", msg::TEXT_RECEIVED);
        self.inner.events.publish(NodeEvent::ClipboardReceived { from_device_id: phone_id(session_id), text });
        Ok(())
    }

    pub(super) fn board(&self) -> &BoardService {
        &self.inner.board
    }

    /// Tells every connected phone to re-fetch the board.
    pub(super) fn announce_board(&self) {
        lock(&self.inner.sessions).notify_all(MobileEvent::Board);
    }

    pub(super) fn inbox(&self, session_id: &str) -> Vec<InboxView> {
        lock(&self.inner.sessions).inbox(session_id, unix_now())
    }

    pub(super) fn inbox_item(&self, session_id: &str, item_id: &str) -> Option<InboxItem> {
        lock(&self.inner.sessions).inbox_item(session_id, item_id, unix_now())
    }

    pub(super) fn latest_text(&self, session_id: &str) -> Option<SharedText> {
        lock(&self.inner.sessions).text(session_id)
    }

    pub(super) fn subscribe(&self, session_id: &str) -> Option<broadcast::Receiver<MobileEvent>> {
        lock(&self.inner.sessions).subscribe(session_id)
    }
}

fn phone_id(session_id: &str) -> String {
    format!("{PHONE_ID_PREFIX}{session_id}")
}
