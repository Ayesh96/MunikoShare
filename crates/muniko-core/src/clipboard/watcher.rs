use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use super::access::ClipboardAccess;
use super::auto_sender::AutoSender;
use super::change_detector::{Change, ChangeDetector};
use super::content::ClipboardContent;
use super::history::HistorySource;
use super::sensitivity::{PlatformProbe, SensitivityProbe};
use super::shared::ClipboardShared;
use crate::constants::clipboard::WATCH_INTERVAL;
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::events::EventBus;
use crate::identity::Identity;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::clipboard as msg;
use crate::trust::TrustStore;
use crate::util::peers::PeerDirectory;
use crate::util::task::BackgroundTask;

/// Polls the local clipboard and mirrors real changes to peers that have clipboard sync enabled.
/// Start it after the HTTP server; stop it before the server.
pub struct ClipboardWatcher {
    worker: Option<Worker>,
    interval: Duration,
    task: Option<BackgroundTask>,
    state: LifecycleState,
}

impl ClipboardWatcher {
    pub fn new(
        identity: Arc<Identity>,
        access: Arc<dyn ClipboardAccess>,
        trust: Arc<dyn TrustStore>,
        peers: Arc<dyn PeerDirectory>,
        events: EventBus,
        shared: ClipboardShared,
    ) -> Self {
        let worker = Worker {
            access,
            trust,
            peers,
            shared,
            probe: Arc::new(PlatformProbe),
            sender: AutoSender::new(identity, events),
            detector: ChangeDetector::default(),
        };
        Self { worker: Some(worker), interval: WATCH_INTERVAL, task: None, state: LifecycleState::Created }
    }

    /// Replaces the OS sensitivity probe (tests, custom policies).
    pub fn with_probe(mut self, probe: Arc<dyn SensitivityProbe>) -> Self {
        if let Some(worker) = self.worker.as_mut() {
            worker.probe = probe;
        }
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

#[async_trait]
impl Lifecycle for ClipboardWatcher {
    async fn start(&mut self) -> Result<()> {
        let mut worker = self.worker.take().ok_or_else(|| MunikoError::InvalidState(msg::WATCHER_ALREADY_RUNNING.to_string()))?;
        let interval = self.interval;
        self.task = Some(BackgroundTask::spawn(move |cancel| async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = worker.tick() => {}
                }
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = tokio::time::sleep(interval) => {}
                }
            }
        }));
        self.state = LifecycleState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(mut task) = self.task.take() {
            task.stop().await;
        }
        self.state = LifecycleState::Stopped;
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

struct Worker {
    access: Arc<dyn ClipboardAccess>,
    trust: Arc<dyn TrustStore>,
    peers: Arc<dyn PeerDirectory>,
    shared: ClipboardShared,
    probe: Arc<dyn SensitivityProbe>,
    sender: AutoSender,
    detector: ChangeDetector,
}

impl Worker {
    async fn tick(&mut self) {
        let targets = self.sync_targets();
        if targets.is_empty() {
            self.detector.go_idle();
            return;
        }
        let Some(content) = self.read_shareable().await else { return };
        let hash = content.hash();
        if self.detector.observe(&hash) != Change::Changed || self.shared.hashes.take_received(&hash) {
            return;
        }
        self.shared.history.record(HistorySource::Local, content.clone());
        self.sender.send_all(&targets, &content).await;
    }

    fn sync_targets(&self) -> Vec<DeviceInfo> {
        self.peers.peers().into_iter().filter(|peer| self.trust.clipboard_sync(&peer.id)).collect()
    }

    /// Current content unless it is sensitive, empty, unreadable or over the limits.
    async fn read_shareable(&self) -> Option<ClipboardContent> {
        let (access, probe) = (self.access.clone(), self.probe.clone());
        let read = tokio::task::spawn_blocking(move || if probe.is_sensitive() { Ok(None) } else { access.read_content() }).await.ok()?;
        match read {
            Ok(Some(content)) => match content.problem() {
                None => Some(content),
                Some(problem) => {
                    tracing::debug!(reason = problem.message(), "{}", msg::READ_SKIPPED);
                    None
                }
            },
            Ok(None) => None,
            Err(error) => {
                tracing::debug!(%error, "{}", msg::READ_SKIPPED);
                None
            }
        }
    }
}
