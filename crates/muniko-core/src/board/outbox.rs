use std::sync::Mutex;

use tokio::sync::Notify;

use super::model::BoardItem;

/// Locally created items waiting to be pushed to peers; the syncer drains it.
#[derive(Default)]
pub struct Outbox {
    pending: Mutex<Vec<BoardItem>>,
    wake: Notify,
}

impl Outbox {
    pub fn push(&self, item: BoardItem) {
        self.pending.lock().unwrap_or_else(|e| e.into_inner()).push(item);
        self.wake.notify_one();
    }

    pub fn take(&self) -> Vec<BoardItem> {
        std::mem::take(&mut *self.pending.lock().unwrap_or_else(|e| e.into_inner()))
    }

    pub async fn wait(&self) {
        self.wake.notified().await;
    }
}
