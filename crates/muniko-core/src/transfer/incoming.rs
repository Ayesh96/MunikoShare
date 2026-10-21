//! State of offers waiting for a decision and of accepted transfers still receiving files.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use tokio::sync::oneshot;

use super::folder_roots::FolderRoots;
use super::manifest::{Manifest, ManifestFile, ReceivedStatus};
use super::partial::PartialStore;
use super::rejection::Rejection;
use crate::error::MunikoError;
use crate::events::{EventBus, NodeEvent};
use crate::messages::transfer as msg;

/// Last time bytes moved for a transfer; shared with in-flight uploads.
pub struct Activity(Mutex<Instant>);

impl Activity {
    fn now() -> Arc<Self> {
        Arc::new(Self(Mutex::new(Instant::now())))
    }

    pub fn touch(&self) {
        *lock(&self.0) = Instant::now();
    }

    fn idle_for(&self) -> Duration {
        lock(&self.0).elapsed()
    }
}

/// Permission to receive one file of an accepted transfer.
pub struct FileTicket {
    pub transfer_id: String,
    pub device_id: String,
    pub index: usize,
    pub file: ManifestFile,
    /// Bytes of the transfer's other files that are already complete.
    pub base_bytes: u64,
    pub total_bytes: u64,
    pub activity: Arc<Activity>,
    pub roots: Arc<FolderRoots>,
}

struct AcceptedTransfer {
    device_id: String,
    files: Vec<ManifestFile>,
    claimed: Vec<bool>,
    done: Vec<bool>,
    completed_bytes: u64,
    total_bytes: u64,
    activity: Arc<Activity>,
    roots: Arc<FolderRoots>,
}

#[derive(Default)]
struct State {
    pending: HashMap<String, oneshot::Sender<bool>>,
    accepted: HashMap<String, AcceptedTransfer>,
}

/// How an offer ended for the sender.
pub enum OfferOutcome {
    Accepted { transfer_id: String },
    Declined,
}

pub struct IncomingTransfers {
    state: Mutex<State>,
    events: EventBus,
    download_dir: PathBuf,
    partials: PartialStore,
    offer_timeout: Duration,
    idle_timeout: Duration,
}

/// Publishes OfferClosed and forgets the offer even if the request is cancelled.
struct OfferGuard<'a> {
    transfers: &'a IncomingTransfers,
    offer_id: String,
    accepted: bool,
}

impl Drop for OfferGuard<'_> {
    fn drop(&mut self) {
        lock(&self.transfers.state).pending.remove(&self.offer_id);
        self.transfers.events.publish(NodeEvent::OfferClosed { offer_id: self.offer_id.clone(), accepted: self.accepted });
    }
}

impl IncomingTransfers {
    pub fn new(events: EventBus, download_dir: PathBuf, offer_timeout: Duration, idle_timeout: Duration) -> Self {
        let partials = PartialStore::new(download_dir.clone());
        Self { state: Mutex::new(State::default()), events, download_dir, partials, offer_timeout, idle_timeout }
    }

    /// Publishes the offer and waits for the user (or accepts at once); the offer id doubles as the transfer id.
    pub async fn offer(&self, device_id: &str, manifest: Manifest, auto_accept: bool) -> std::result::Result<OfferOutcome, Rejection> {
        let offer_id = uuid::Uuid::new_v4().simple().to_string();
        let (decide, decision) = oneshot::channel();
        if !auto_accept {
            lock(&self.state).pending.insert(offer_id.clone(), decide);
        }
        let mut guard = OfferGuard { transfers: self, offer_id: offer_id.clone(), accepted: false };
        self.events.publish(NodeEvent::IncomingOffer {
            offer_id: offer_id.clone(),
            from_device_id: device_id.to_string(),
            files: manifest.summaries(),
            total_size: manifest.total_size(),
        });
        let answer = if auto_accept {
            tracing::info!("{}", msg::OFFER_AUTO_ACCEPTED_LOG);
            Ok(Ok(true))
        } else {
            tokio::time::timeout(self.offer_timeout, decision).await
        };
        match answer {
            Err(_) => Err(Rejection::OfferTimedOut),
            Ok(Ok(true)) => {
                self.register(&offer_id, device_id, manifest);
                guard.accepted = true;
                Ok(OfferOutcome::Accepted { transfer_id: offer_id })
            }
            Ok(_) => Ok(OfferOutcome::Declined),
        }
    }

    /// Delivers the user's decision to the waiting offer request.
    pub fn respond(&self, offer_id: &str, accept: bool) -> crate::error::Result<()> {
        let decide = lock(&self.state).pending.remove(offer_id);
        let unknown = || MunikoError::Transfer(msg::UNKNOWN_OFFER.to_string());
        decide.ok_or_else(unknown)?.send(accept).map_err(|_| unknown())
    }

    /// Reserves file `index` for `device_id`; only the offering device may upload.
    pub fn claim_file(&self, transfer_id: &str, device_id: &str, index: usize) -> std::result::Result<FileTicket, Rejection> {
        let mut state = lock(&self.state);
        self.sweep(&mut state);
        let transfer = state.accepted.get_mut(transfer_id).ok_or(Rejection::NotFound(msg::UNKNOWN_TRANSFER))?;
        if transfer.device_id != device_id {
            return Err(Rejection::Forbidden(msg::WRONG_DEVICE));
        }
        let file = transfer.files.get(index).cloned().ok_or(Rejection::NotFound(msg::UNKNOWN_FILE))?;
        if transfer.claimed[index] {
            return Err(Rejection::Conflict(msg::FILE_ALREADY_SENT));
        }
        transfer.claimed[index] = true;
        transfer.activity.touch();
        Ok(FileTicket {
            transfer_id: transfer_id.to_string(),
            device_id: device_id.to_string(),
            index,
            file,
            base_bytes: transfer.completed_bytes,
            total_bytes: transfer.total_bytes,
            activity: transfer.activity.clone(),
            roots: transfer.roots.clone(),
        })
    }

    /// How much of file `index` is stored; lets the offering device resume an interrupted upload.
    pub async fn received_status(&self, transfer_id: &str, device_id: &str, index: usize) -> std::result::Result<ReceivedStatus, Rejection> {
        let (size, done) = {
            let mut state = lock(&self.state);
            self.sweep(&mut state);
            let transfer = state.accepted.get(transfer_id).ok_or(Rejection::NotFound(msg::UNKNOWN_TRANSFER))?;
            if transfer.device_id != device_id {
                return Err(Rejection::Forbidden(msg::WRONG_DEVICE));
            }
            let file = transfer.files.get(index).ok_or(Rejection::NotFound(msg::UNKNOWN_FILE))?;
            if transfer.claimed[index] && !transfer.done[index] {
                return Err(Rejection::Conflict(msg::FILE_BUSY));
            }
            transfer.activity.touch();
            (file.size, transfer.done[index])
        };
        let received = if done { size } else { self.partials.len(transfer_id, index).await };
        Ok(ReceivedStatus { received, complete: done })
    }

    /// Lets the sender retry a file whose upload failed.
    pub fn release_file(&self, ticket: &FileTicket) {
        if let Some(transfer) = lock(&self.state).accepted.get_mut(&ticket.transfer_id) {
            transfer.claimed[ticket.index] = false;
        }
    }

    /// Records a verified file and forgets the transfer once every file arrived.
    pub fn complete_file(&self, ticket: &FileTicket) {
        let mut state = lock(&self.state);
        let Some(transfer) = state.accepted.get_mut(&ticket.transfer_id) else { return };
        transfer.done[ticket.index] = true;
        transfer.completed_bytes += ticket.file.size;
        transfer.activity.touch();
        if transfer.done.iter().all(|done| *done) {
            state.accepted.remove(&ticket.transfer_id);
        }
    }

    fn register(&self, transfer_id: &str, device_id: &str, manifest: Manifest) {
        let count = manifest.files.len();
        let transfer = AcceptedTransfer {
            device_id: device_id.to_string(),
            total_bytes: manifest.total_size(),
            files: manifest.files,
            claimed: vec![false; count],
            done: vec![false; count],
            completed_bytes: 0,
            activity: Activity::now(),
            roots: Arc::new(FolderRoots::new(self.download_dir.clone())),
        };
        let mut state = lock(&self.state);
        self.sweep(&mut state);
        state.accepted.insert(transfer_id.to_string(), transfer);
    }

    // Lazy expiry: no background task to stop, stale entries and their partials go on the next request.
    fn sweep(&self, state: &mut State) {
        let idle_timeout = self.idle_timeout;
        let expired: Vec<String> =
            state.accepted.iter().filter(|(_, transfer)| transfer.activity.idle_for() >= idle_timeout).map(|(id, _)| id.clone()).collect();
        for id in expired {
            if let Some(transfer) = state.accepted.remove(&id) {
                self.partials.discard_transfer(&id, transfer.files.len());
            }
        }
    }
}

// A poisoned lock only means another request panicked; the data is still usable.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
