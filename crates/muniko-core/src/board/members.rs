use std::collections::HashMap;

use super::model::{BoardItem, ItemKind};
use crate::util::fingerprint::fingerprint;

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub cert_der: Vec<u8>,
}

/// Verified members of a board, built from its Member items.
#[derive(Debug, Default)]
pub struct MemberDirectory {
    members: HashMap<String, Member>,
}

/// The certificate a Member item carries, if it belongs to the item's author.
pub fn self_certified(item: &BoardItem) -> Option<&[u8]> {
    match &item.kind {
        ItemKind::Member { device_id, cert_der, .. } if *device_id == item.author_id && fingerprint(cert_der) == *device_id => Some(cert_der),
        _ => None,
    }
}

impl MemberDirectory {
    pub fn from_items(items: &[BoardItem]) -> Self {
        let mut directory = Self::default();
        items.iter().for_each(|item| directory.add(item));
        directory
    }

    /// Records a Member item that was already verified.
    pub fn add(&mut self, item: &BoardItem) {
        if let ItemKind::Member { device_id, name, cert_der } = &item.kind {
            self.members.entry(device_id.clone()).or_insert_with(|| Member { name: name.clone(), cert_der: cert_der.clone() });
        }
    }

    pub fn contains(&self, device_id: &str) -> bool {
        self.members.contains_key(device_id)
    }

    pub fn get(&self, device_id: &str) -> Option<&Member> {
        self.members.get(device_id)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}
