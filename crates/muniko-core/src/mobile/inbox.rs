//! Desktop files offered to one phone.

use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct InboxItem {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub added_at: u64,
}

impl InboxItem {
    pub fn view(&self) -> InboxView {
        InboxView { id: self.id.clone(), name: self.name.clone(), size: self.size, added_at: self.added_at }
    }
}

/// What the phone sees of an inbox item; the desktop path stays private.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InboxView {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub added_at: u64,
}
