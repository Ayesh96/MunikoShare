use std::collections::HashMap;

use serde::Serialize;

use super::files::Availability;
use super::members::MemberDirectory;
use super::model::{BoardItem, ItemKind};
use crate::messages::board::UNKNOWN_AUTHOR;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ViewKind {
    Text { text: String },
    File { name: String, size: u64, sha256: String },
}

/// An item as the UI shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardItemView {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub created_at: u64,
    pub mine: bool,
    pub kind: ViewKind,
    /// This device has the content (always true for text).
    pub held_locally: bool,
    /// Held here or by an online member; false means "author offline" for a file nobody online has.
    pub available: bool,
}

/// Visible items, newest first; deleted items and bookkeeping records are left out.
pub fn build_views(items: Vec<BoardItem>, members: &MemberDirectory, tombstones: &HashMap<String, String>, local_id: &str, availability: &Availability) -> Vec<BoardItemView> {
    let mut views: Vec<BoardItemView> = items
        .into_iter()
        .filter(|item| tombstones.get(&item.id) != Some(&item.author_id))
        .filter_map(|item| view_of(item, members, local_id, availability))
        .collect();
    views.sort_by(|a, b| (b.created_at, &b.id).cmp(&(a.created_at, &a.id)));
    views
}

pub fn view_of(item: BoardItem, members: &MemberDirectory, local_id: &str, availability: &Availability) -> Option<BoardItemView> {
    let (held_locally, available) = availability.status(&item.kind);
    let kind = match item.kind {
        ItemKind::Text { text } => ViewKind::Text { text },
        ItemKind::File { name, size, sha256 } => ViewKind::File { name, size, sha256 },
        ItemKind::Tombstone { .. } | ItemKind::Member { .. } => return None,
    };
    let author_name = members.get(&item.author_id).map(|member| member.name.clone()).unwrap_or_else(|| UNKNOWN_AUTHOR.to_string());
    Some(BoardItemView { mine: item.author_id == local_id, id: item.id, author_id: item.author_id, author_name, created_at: item.created_at, kind, held_locally, available })
}
