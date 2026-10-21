//! What a phone's live stream can be told.

use crate::constants::mobile::{EVENT_BOARD, EVENT_INBOX, EVENT_REVOKED, EVENT_TEXT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileEvent {
    Inbox,
    Text,
    Board,
    Revoked,
}

impl MobileEvent {
    pub fn name(self) -> &'static str {
        match self {
            Self::Inbox => EVENT_INBOX,
            Self::Text => EVENT_TEXT,
            Self::Board => EVENT_BOARD,
            Self::Revoked => EVENT_REVOKED,
        }
    }
}
