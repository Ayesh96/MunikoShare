use std::collections::HashMap;
use std::sync::Mutex;

use super::model::BoardInfo;

/// A board seen on a peer; shown when this device has no board or a different one.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservedBoard {
    pub peer_id: String,
    pub peer_name: String,
    pub info: BoardInfo,
}

/// What each peer last reported about its board.
#[derive(Default)]
pub struct ObservedBoards {
    by_peer: Mutex<HashMap<String, ObservedBoard>>,
}

impl ObservedBoards {
    /// Stores the latest report for a peer; true when it differs from the previous one.
    pub fn record(&self, peer_id: &str, peer_name: &str, info: Option<BoardInfo>) -> bool {
        let mut map = self.by_peer.lock().unwrap_or_else(|e| e.into_inner());
        match info {
            Some(info) => {
                let observed = ObservedBoard { peer_id: peer_id.to_string(), peer_name: peer_name.to_string(), info };
                map.insert(peer_id.to_string(), observed.clone()) != Some(observed)
            }
            None => map.remove(peer_id).is_some(),
        }
    }

    pub fn forget_except(&self, present: &[String]) -> bool {
        let mut map = self.by_peer.lock().unwrap_or_else(|e| e.into_inner());
        let before = map.len();
        map.retain(|peer_id, _| present.contains(peer_id));
        map.len() != before
    }

    /// Boards other than `local_id` (all of them when it is `None`), one per board id.
    pub fn others(&self, local_id: Option<&str>) -> Vec<ObservedBoard> {
        let map = self.by_peer.lock().unwrap_or_else(|e| e.into_inner());
        let mut seen = std::collections::HashSet::new();
        let mut boards: Vec<ObservedBoard> = map.values().filter(|b| Some(b.info.board.id.as_str()) != local_id).cloned().collect();
        boards.sort_by(|a, b| a.peer_id.cmp(&b.peer_id));
        boards.retain(|b| seen.insert(b.info.board.id.clone()));
        boards
    }
}
