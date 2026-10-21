use super::model::ItemKind;
use crate::constants::board::*;
use crate::util::hash::is_sha256_hex;

/// Size and shape checks every replica applies to items it did not create itself.
pub fn within_limits(kind: &ItemKind) -> bool {
    match kind {
        ItemKind::Text { text } => !text.is_empty() && text.len() <= MAX_TEXT_BYTES,
        ItemKind::File { name, sha256, .. } => valid_file_name(name) && is_sha256_hex(sha256),
        ItemKind::Tombstone { target_id } => !target_id.is_empty(),
        ItemKind::Member { name, .. } => name.chars().count() <= MAX_MEMBER_NAME_CHARS,
    }
}

pub fn valid_file_name(name: &str) -> bool {
    !name.is_empty() && name.chars().count() <= MAX_FILE_NAME_CHARS && !name.contains(['/', '\\'])
}
