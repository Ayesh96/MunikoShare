use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;

use super::{clipboard_preview, HistoryKind, HistoryStore, NewHistoryEntry};
use crate::error::{MunikoError, Result};
use crate::events::{EventBus, NodeEvent};
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::history as msg;
use crate::util::task::BackgroundTask;

/// Turns node events into history entries; None for events not worth recording.
fn entry_for(event: NodeEvent) -> Option<NewHistoryEntry> {
    let entry = match event {
        NodeEvent::FileReceived { from_device_id, path } => NewHistoryEntry::new(HistoryKind::FileReceived, from_device_id, path.to_string_lossy()),
        NodeEvent::FileSent { to_device_id, path } => NewHistoryEntry::new(HistoryKind::FileSent, to_device_id, path.to_string_lossy()),
        NodeEvent::TransferFailed { device_id, reason } => NewHistoryEntry::new(HistoryKind::TransferFailed, device_id, reason),
        NodeEvent::ClipboardReceived { from_device_id, text } => {
            NewHistoryEntry::new(HistoryKind::ClipboardReceived, from_device_id, clipboard_preview(&text))
        }
        NodeEvent::ClipboardSent { to_device_id } => NewHistoryEntry::new(HistoryKind::ClipboardSent, to_device_id, ""),
        NodeEvent::PairingCompleted { device_id, device_name } => NewHistoryEntry::new(HistoryKind::Paired, device_id, device_name),
        NodeEvent::DeviceUnpaired { device_id } => NewHistoryEntry::new(HistoryKind::Unpaired, device_id, ""),
        _ => return None,
    };
    Some(entry)
}

async fn record_blocking(store: &Arc<dyn HistoryStore>, entry: NewHistoryEntry) {
    let store = Arc::clone(store);
    match tokio::task::spawn_blocking(move || store.record(entry)).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => tracing::warn!("{}: {error}", msg::RECORD_FAILED),
        Err(error) => tracing::warn!("{}: {error}", msg::RECORD_TASK_FAILED),
    }
}

async fn run(store: Arc<dyn HistoryStore>, mut events: tokio::sync::broadcast::Receiver<NodeEvent>, cancel: CancellationToken) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            received = events.recv() => match received {
                Ok(event) => {
                    if let Some(entry) = entry_for(event) {
                        record_blocking(&store, entry).await;
                    }
                }
                Err(RecvError::Lagged(missed)) => tracing::warn!("{}: {missed}", msg::EVENTS_LAGGED),
                Err(RecvError::Closed) => break,
            },
        }
    }
}

/// Records node events into a HistoryStore while running.
pub struct HistoryRecorder {
    store: Arc<dyn HistoryStore>,
    events: EventBus,
    state: LifecycleState,
    task: Option<BackgroundTask>,
}

impl HistoryRecorder {
    pub fn new(store: Arc<dyn HistoryStore>, events: EventBus) -> Self {
        Self { store, events, state: LifecycleState::Created, task: None }
    }
}

#[async_trait]
impl Lifecycle for HistoryRecorder {
    async fn start(&mut self) -> Result<()> {
        if self.state != LifecycleState::Created {
            return Err(MunikoError::InvalidState(format!("{:?}", self.state)));
        }
        let (store, events) = (Arc::clone(&self.store), self.events.subscribe());
        self.task = Some(BackgroundTask::spawn(|cancel| run(store, events, cancel)));
        self.state = LifecycleState::Running;
        tracing::info!("{}", msg::RECORDER_STARTED);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(mut task) = self.task.take() {
            task.stop().await;
            tracing::info!("{}", msg::RECORDER_STOPPED);
        }
        self.state = LifecycleState::Stopped;
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;
    use crate::history::MemoryHistoryStore;

    async fn wait_for(store: &MemoryHistoryStore, count: usize) -> Vec<crate::history::HistoryEntry> {
        for _ in 0..100 {
            let entries = store.recent(100).unwrap();
            if entries.len() >= count {
                return entries;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        store.recent(100).unwrap()
    }

    fn fixture() -> (Arc<MemoryHistoryStore>, EventBus, HistoryRecorder) {
        let store = Arc::new(MemoryHistoryStore::new());
        let bus = EventBus::new();
        let recorder = HistoryRecorder::new(store.clone(), bus.clone());
        (store, bus, recorder)
    }

    #[tokio::test]
    async fn records_mapped_events_and_ignores_others() {
        let (store, bus, mut recorder) = fixture();
        recorder.start().await.unwrap();
        bus.publish(NodeEvent::PeerLost { device_id: "x".into() });
        bus.publish(NodeEvent::FileReceived { from_device_id: "a".into(), path: PathBuf::from("f.txt") });
        bus.publish(NodeEvent::ClipboardReceived { from_device_id: "a".into(), text: "é".repeat(500) });
        bus.publish(NodeEvent::PairingCompleted { device_id: "a".into(), device_name: "Laptop".into() });
        bus.publish(NodeEvent::DeviceUnpaired { device_id: "a".into() });
        let entries = wait_for(&store, 4).await;
        recorder.stop().await.unwrap();
        let kinds: Vec<_> = entries.iter().map(|e| e.kind).collect();
        assert_eq!(kinds, [HistoryKind::Unpaired, HistoryKind::Paired, HistoryKind::ClipboardReceived, HistoryKind::FileReceived]);
        assert_eq!(entries[2].detail.chars().count(), crate::constants::history::CLIPBOARD_PREVIEW_CHARS);
        assert_eq!(entries[3].detail, "f.txt");
    }

    #[tokio::test]
    async fn stop_ends_task_and_start_twice_fails() {
        let (store, bus, mut recorder) = fixture();
        recorder.start().await.unwrap();
        assert!(matches!(recorder.start().await, Err(MunikoError::InvalidState(_))));
        recorder.stop().await.unwrap();
        assert_eq!(recorder.state(), LifecycleState::Stopped);
        bus.publish(NodeEvent::TransferFailed { device_id: "a".into(), reason: "x".into() });
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(store.recent(10).unwrap().is_empty());
    }
}
