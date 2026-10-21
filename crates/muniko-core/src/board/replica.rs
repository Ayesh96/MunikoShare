use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use super::limits::within_limits;
use super::members::{self_certified, MemberDirectory};
use super::model::{BoardInfo, BoardItem, BoardRecord, DigestEntry, ItemKind};
use super::signing::verify_item;
use super::store::BoardStore;
use crate::constants::board::{KIND_MEMBER, KIND_TOMBSTONE};
use crate::board::BoardError;
use crate::error::{MunikoError, Result};

/// Outcome of offering items to the replica.
#[derive(Debug, Default)]
pub struct Accepted {
    pub added: Vec<BoardItem>,
    pub rejected: usize,
}

enum Verdict {
    Stored,
    Skipped,
    Rejected,
}

/// This device's copy of the board: verifies everything it stores and applies tombstones.
pub struct Replica {
    store: Arc<dyn BoardStore>,
    retention: Duration,
}

impl Replica {
    pub fn new(store: Arc<dyn BoardStore>, retention: Duration) -> Self {
        Self { store, retention }
    }

    pub fn store(&self) -> &dyn BoardStore {
        self.store.as_ref()
    }

    pub fn record(&self) -> Result<Option<BoardRecord>> {
        self.store.load_board()
    }

    pub fn require_record(&self) -> Result<BoardRecord> {
        self.record()?.ok_or_else(|| MunikoError::from(BoardError::NoBoard))
    }

    pub fn members(&self) -> Result<MemberDirectory> {
        Ok(MemberDirectory::from_items(&self.store.by_kind(KIND_MEMBER)?))
    }

    pub fn info(&self) -> Result<Option<BoardInfo>> {
        let Some(record) = self.record()? else { return Ok(None) };
        Ok(Some(BoardInfo { board: record.board, member_count: self.members()?.len() }))
    }

    fn cutoff(&self, now: u64) -> u64 {
        now.saturating_sub(self.retention.as_secs())
    }

    /// Verifies and stores items; Member items go first so a batch can introduce its own authors.
    pub fn accept(&self, items: Vec<BoardItem>, now: u64) -> Result<Accepted> {
        let record = self.require_record()?;
        let mut members = self.members()?;
        let mut tombstones = self.tombstone_authors()?;
        let cutoff = self.cutoff(now);
        let (member_items, others): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.kind.tag() == KIND_MEMBER);
        let mut accepted = Accepted::default();
        for item in member_items.into_iter().chain(others) {
            match self.admit(&record.board.id, &mut members, &mut tombstones, &item, cutoff)? {
                Verdict::Stored => accepted.added.push(item),
                Verdict::Skipped => {}
                Verdict::Rejected => accepted.rejected += 1,
            }
        }
        Ok(accepted)
    }

    fn admit(
        &self,
        board_id: &str,
        members: &mut MemberDirectory,
        tombstones: &mut HashMap<String, String>,
        item: &BoardItem,
        cutoff: u64,
    ) -> Result<Verdict> {
        if item.board_id != board_id || !within_limits(&item.kind) || !Self::signature_valid(members, item) {
            return Ok(Verdict::Rejected);
        }
        let is_member = item.kind.tag() == KIND_MEMBER;
        let expired = item.created_at < cutoff && !is_member;
        let tombstoned = tombstones.get(&item.id) == Some(&item.author_id);
        if expired || tombstoned || !self.store.insert(item)? {
            return Ok(Verdict::Skipped);
        }
        match &item.kind {
            ItemKind::Member { .. } => members.add(item),
            ItemKind::Tombstone { target_id } => {
                tombstones.insert(target_id.clone(), item.author_id.clone());
                self.drop_target(target_id, &item.author_id)?;
            }
            _ => {}
        }
        Ok(Verdict::Stored)
    }

    fn signature_valid(members: &MemberDirectory, item: &BoardItem) -> bool {
        if item.kind.tag() == KIND_MEMBER {
            return self_certified(item).is_some_and(|cert| verify_item(item, cert));
        }
        members.get(&item.author_id).is_some_and(|member| verify_item(item, &member.cert_der))
    }

    /// Drops a deleted item's content; only its author can delete it and members are permanent.
    fn drop_target(&self, target_id: &str, tombstone_author: &str) -> Result<()> {
        match self.store.get(target_id)? {
            Some(target) if target.author_id == tombstone_author && target.kind.tag() != KIND_MEMBER => self.store.delete(target_id),
            _ => Ok(()),
        }
    }

    /// Map of deleted item id to the author of its tombstone.
    pub fn tombstone_authors(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        for item in self.store.by_kind(KIND_TOMBSTONE)? {
            if let ItemKind::Tombstone { target_id } = item.kind {
                map.insert(target_id, item.author_id);
            }
        }
        Ok(map)
    }

    /// Ids from a peer's digest that this replica should fetch.
    pub fn missing(&self, remote: &[DigestEntry], now: u64) -> Result<Vec<String>> {
        let local: HashSet<String> = self.store.digest()?.into_iter().map(|entry| entry.id).collect();
        let deleted = self.tombstone_authors()?;
        let cutoff = self.cutoff(now);
        let wanted = |entry: &&DigestEntry| !local.contains(&entry.id) && !deleted.contains_key(&entry.id) && entry.created_at >= cutoff;
        Ok(remote.iter().filter(wanted).map(|entry| entry.id.clone()).collect())
    }

    pub fn prune(&self, now: u64) -> Result<usize> {
        self.store.prune_before(self.cutoff(now))
    }
}
