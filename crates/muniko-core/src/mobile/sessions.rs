//! All live phone sessions.

use std::collections::HashMap;

use tokio::sync::broadcast;

use super::event::MobileEvent;
use super::inbox::{InboxItem, InboxView};
use super::session::{PhoneSession, SessionEntry, SharedText};
use super::secret::random_secret;
use crate::error::Result;

pub struct SessionStore {
    entries: HashMap<String, SessionEntry>,
    session_ttl_secs: u64,
    inbox_ttl_secs: u64,
}

impl SessionStore {
    pub fn new(session_ttl_secs: u64, inbox_ttl_secs: u64) -> Self {
        Self { entries: HashMap::new(), session_ttl_secs, inbox_ttl_secs }
    }

    pub fn create(&mut self, label: String, now: u64) -> Result<PhoneSession> {
        let info = PhoneSession { id: random_secret()?, label, connected_at: now, last_seen: now };
        self.entries.insert(info.id.clone(), SessionEntry::new(info.clone()));
        Ok(info)
    }

    /// Slides the expiry and returns the session, or None when unknown or expired.
    pub fn authenticate(&mut self, id: &str, now: u64) -> Option<PhoneSession> {
        let entry = self.entries.get_mut(id).filter(|entry| !entry.is_expired(now, self.session_ttl_secs))?;
        entry.info.last_seen = now;
        Some(entry.info.clone())
    }

    pub fn list(&self) -> Vec<PhoneSession> {
        let mut sessions: Vec<PhoneSession> = self.entries.values().map(|entry| entry.info.clone()).collect();
        sessions.sort_by_key(|session| session.connected_at);
        sessions
    }

    /// Ends the session after telling its open stream.
    pub fn remove(&mut self, id: &str) -> bool {
        let Some(entry) = self.entries.remove(id) else { return false };
        entry.notify(MobileEvent::Revoked);
        true
    }

    pub fn remove_all(&mut self) -> Vec<String> {
        let ids: Vec<String> = self.entries.keys().cloned().collect();
        ids.into_iter().filter(|id| self.remove(id)).collect()
    }

    /// Removes sessions past their TTL and returns their ids; also prunes old inbox items.
    pub fn expire(&mut self, now: u64) -> Vec<String> {
        let ttl = self.session_ttl_secs;
        let expired: Vec<String> = self.entries.iter().filter(|(_, entry)| entry.is_expired(now, ttl)).map(|(id, _)| id.clone()).collect();
        let removed = expired.into_iter().filter(|id| self.remove(id)).collect();
        for entry in self.entries.values_mut() {
            entry.drop_expired_items(now, self.inbox_ttl_secs);
        }
        removed
    }

    pub fn notify_all(&self, event: MobileEvent) {
        self.entries.values().for_each(|entry| entry.notify(event));
    }

    pub fn add_inbox(&mut self, id: &str, item: InboxItem) -> bool {
        let Some(entry) = self.entries.get_mut(id) else { return false };
        entry.inbox.push(item);
        entry.notify(MobileEvent::Inbox);
        true
    }

    /// Unexpired inbox items, newest first.
    pub fn inbox(&self, id: &str, now: u64) -> Vec<InboxView> {
        let items = self.live_items(id, now);
        items.into_iter().rev().map(InboxItem::view).collect()
    }

    pub fn inbox_item(&self, id: &str, item_id: &str, now: u64) -> Option<InboxItem> {
        self.live_items(id, now).into_iter().find(|item| item.id == item_id).cloned()
    }

    pub fn set_text(&mut self, id: &str, text: String, now: u64) -> bool {
        let Some(entry) = self.entries.get_mut(id) else { return false };
        entry.text = Some(SharedText { text, sent_at: now });
        entry.notify(MobileEvent::Text);
        true
    }

    pub fn text(&self, id: &str) -> Option<SharedText> {
        self.entries.get(id)?.text.clone()
    }

    pub fn subscribe(&self, id: &str) -> Option<broadcast::Receiver<MobileEvent>> {
        self.entries.get(id).map(SessionEntry::subscribe)
    }

    fn live_items(&self, id: &str, now: u64) -> Vec<&InboxItem> {
        let ttl = self.inbox_ttl_secs;
        self.entries.get(id).map(|entry| entry.inbox.iter().filter(|item| now < item.added_at + ttl).collect()).unwrap_or_default()
    }
}
