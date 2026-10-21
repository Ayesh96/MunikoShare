//! The phone companion facade: invites, sessions, inbox and text for both the desktop UI and the routes.

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use uuid::Uuid;

use super::attempts::AttemptLimiter;
use super::inbox::InboxItem;
use super::limits::MobileLimits;
use super::session::PhoneSession;
use super::sessions::SessionStore;
use super::tokens::InviteTokens;
use super::user_agent::label_from_user_agent;
use crate::board::BoardService;
use crate::clipboard::ClipboardAccess;
use crate::constants::mobile::INVITE_FRAGMENT;
use crate::constants::mobile::ROUTE_MOBILE_DIR;
use crate::error::{MunikoError, Result};
use crate::events::{EventBus, NodeEvent};
use crate::messages::mobile as msg;
use crate::util::lock::lock;
use crate::util::time::unix_now;

/// What the desktop shows as a QR code: one URL per LAN address, all carrying the same token.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    pub urls: Vec<String>,
    pub token: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedeemError {
    Throttled,
    Invalid,
}

pub(super) struct Inner {
    pub(super) device_name: String,
    pub(super) download_dir: PathBuf,
    pub(super) events: EventBus,
    pub(super) clipboard: Arc<dyn ClipboardAccess>,
    pub(super) board: Arc<BoardService>,
    pub(super) limits: MobileLimits,
    pub(super) enabled: AtomicBool,
    pub(super) tokens: Mutex<InviteTokens>,
    pub(super) sessions: Mutex<SessionStore>,
    pub(super) attempts: Mutex<AttemptLimiter>,
}

/// Cheap to clone; every clone shares the same sessions.
#[derive(Clone)]
pub struct MobileService {
    pub(super) inner: Arc<Inner>,
}

impl MobileService {
    pub fn new(device_name: String, download_dir: PathBuf, events: EventBus, clipboard: Arc<dyn ClipboardAccess>, board: Arc<BoardService>) -> Self {
        Self::with_limits(device_name, download_dir, events, clipboard, board, MobileLimits::default())
    }

    pub fn with_limits(device_name: String, download_dir: PathBuf, events: EventBus, clipboard: Arc<dyn ClipboardAccess>, board: Arc<BoardService>, limits: MobileLimits) -> Self {
        let sessions = SessionStore::new(limits.session_ttl_secs, limits.inbox_ttl_secs);
        let inner = Inner {
            device_name,
            download_dir,
            events,
            clipboard,
            board,
            limits,
            enabled: AtomicBool::new(true),
            tokens: Mutex::default(),
            sessions: Mutex::new(sessions),
            attempts: Mutex::default(),
        };
        Self { inner: Arc::new(inner) }
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.enabled.load(Ordering::SeqCst)
    }

    /// Turning it off ends every session and forgets pending invites.
    pub fn set_enabled(&self, enabled: bool) {
        self.inner.enabled.store(enabled, Ordering::SeqCst);
        if !enabled {
            lock(&self.inner.tokens).clear();
            let ended = lock(&self.inner.sessions).remove_all();
            self.announce_disconnected(ended);
        }
    }

    /// `base_urls` look like `https://192.168.1.5:53420`; the token goes in the fragment so it never reaches logs.
    pub fn create_invite(&self, base_urls: Vec<String>) -> Result<Invite> {
        if !self.is_enabled() {
            return Err(MunikoError::InvalidState(msg::MOBILE_DISABLED.to_string()));
        }
        let (token, expires_at) = lock(&self.inner.tokens).issue(unix_now(), self.inner.limits.token_ttl_secs)?;
        let urls = base_urls.iter().map(|base| format!("{}{ROUTE_MOBILE_DIR}{INVITE_FRAGMENT}{token}", base.trim_end_matches('/'))).collect();
        Ok(Invite { urls, token, expires_at })
    }

    pub fn sessions(&self) -> Vec<PhoneSession> {
        lock(&self.inner.sessions).list()
    }

    /// Ends one phone's session; false when it is not (or no longer) connected.
    pub fn revoke(&self, session_id: &str) -> bool {
        let removed = lock(&self.inner.sessions).remove(session_id);
        if removed {
            self.announce_disconnected(vec![session_id.to_string()]);
        }
        removed
    }

    /// Adds regular files to the phone's inbox and returns how many were added.
    pub async fn send_files(&self, session_id: &str, paths: Vec<PathBuf>) -> Result<usize> {
        let mut items = Vec::new();
        for path in paths {
            items.extend(inbox_item_for(path).await?);
        }
        if items.is_empty() {
            return Err(MunikoError::Transfer(msg::NO_FILES.to_string()));
        }
        let count = items.len();
        let mut sessions = lock(&self.inner.sessions);
        for item in items {
            if !sessions.add_inbox(session_id, item) {
                return Err(MunikoError::PeerNotFound(msg::SESSION_NOT_FOUND.to_string()));
            }
        }
        Ok(count)
    }

    /// Makes `text` available to the phone, which is told over its live stream.
    pub fn share_text(&self, session_id: &str, text: String) -> Result<()> {
        if text.is_empty() || !lock(&self.inner.sessions).set_text(session_id, text, unix_now()) {
            return Err(MunikoError::PeerNotFound(msg::SESSION_NOT_FOUND.to_string()));
        }
        Ok(())
    }

    pub(super) fn redeem(&self, token: &str, ip: IpAddr, user_agent: &str) -> std::result::Result<PhoneSession, RedeemError> {
        let (now, limits) = (unix_now(), &self.inner.limits);
        let throttled = lock(&self.inner.attempts).is_blocked(ip, now, limits.redeem_window_secs, limits.max_failed_redeems);
        if throttled {
            return Err(RedeemError::Throttled);
        }
        if !self.is_enabled() || !lock(&self.inner.tokens).redeem(token, now) {
            lock(&self.inner.attempts).record_failure(ip, now);
            tracing::warn!(%ip, "{}", msg::TOKEN_REJECTED);
            return Err(RedeemError::Invalid);
        }
        let session = lock(&self.inner.sessions).create(label_from_user_agent(user_agent), now).map_err(|_| RedeemError::Invalid)?;
        tracing::info!(label = %session.label, "{}", msg::PHONE_CONNECTED);
        self.inner.events.publish(NodeEvent::PhoneConnected { session_id: session.id.clone(), label: session.label.clone() });
        Ok(session)
    }

    pub(super) fn authenticate(&self, session_id: &str) -> Option<PhoneSession> {
        if !self.is_enabled() {
            return None;
        }
        lock(&self.inner.sessions).authenticate(session_id, unix_now())
    }

    pub(super) fn device_name(&self) -> &str {
        &self.inner.device_name
    }

    /// Expires tokens, sessions, inbox items and old rate-limit records.
    pub(super) fn sweep(&self) {
        let (now, limits) = (unix_now(), &self.inner.limits);
        lock(&self.inner.tokens).purge(now);
        lock(&self.inner.attempts).purge(now, limits.redeem_window_secs);
        let expired = lock(&self.inner.sessions).expire(now);
        self.announce_disconnected(expired);
    }

    fn announce_disconnected(&self, session_ids: Vec<String>) {
        for session_id in session_ids {
            tracing::info!("{}", msg::PHONE_DISCONNECTED);
            self.inner.events.publish(NodeEvent::PhoneDisconnected { session_id });
        }
    }
}

// Directories, links to directories and missing paths are skipped, not errors.
async fn inbox_item_for(path: PathBuf) -> Result<Option<InboxItem>> {
    let Ok(metadata) = tokio::fs::metadata(&path).await else { return Ok(None) };
    let name = path.file_name().map(|name| name.to_string_lossy().into_owned());
    match (metadata.is_file(), name) {
        (true, Some(name)) => Ok(Some(InboxItem { id: Uuid::new_v4().simple().to_string(), name, path, size: metadata.len(), added_at: unix_now() })),
        _ => Ok(None),
    }
}
