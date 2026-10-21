//! One phone's session: identity, inbox, latest shared text and live-event channel.

use serde::Serialize;
use tokio::sync::broadcast;

use super::event::MobileEvent;
use super::inbox::InboxItem;
use crate::constants::mobile::MAX_SESSION_EVENTS;

/// A connected phone as the desktop lists it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhoneSession {
    pub id: String,
    pub label: String,
    pub connected_at: u64,
    pub last_seen: u64,
}

#[derive(Debug, Clone)]
pub struct SharedText {
    pub text: String,
    pub sent_at: u64,
}

pub struct SessionEntry {
    pub info: PhoneSession,
    pub inbox: Vec<InboxItem>,
    pub text: Option<SharedText>,
    events: broadcast::Sender<MobileEvent>,
}

impl SessionEntry {
    pub fn new(info: PhoneSession) -> Self {
        let (events, _) = broadcast::channel(MAX_SESSION_EVENTS);
        Self { info, inbox: Vec::new(), text: None, events }
    }

    pub fn is_expired(&self, now: u64, ttl_secs: u64) -> bool {
        now >= self.info.last_seen + ttl_secs
    }

    pub fn notify(&self, event: MobileEvent) {
        // No open stream is fine; the phone catches up on its next fetch.
        let _ = self.events.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MobileEvent> {
        self.events.subscribe()
    }

    pub fn drop_expired_items(&mut self, now: u64, ttl_secs: u64) {
        self.inbox.retain(|item| now < item.added_at + ttl_secs);
    }
}
