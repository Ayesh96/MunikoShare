//! Counts transferred bytes and publishes throttled progress events.

use std::time::Instant;

use crate::constants::transfer::PROGRESS_INTERVAL;
use crate::events::{EventBus, NodeEvent, TransferDirection};

pub struct ProgressReporter {
    events: EventBus,
    transfer_id: String,
    device_id: String,
    direction: TransferDirection,
    bytes_done: u64,
    bytes_total: u64,
    last_published: Option<Instant>,
}

impl ProgressReporter {
    /// `bytes_done` is what earlier files of the same transfer already moved.
    pub fn new(events: EventBus, transfer_id: &str, device_id: &str, direction: TransferDirection, bytes_done: u64, bytes_total: u64) -> Self {
        Self {
            events,
            transfer_id: transfer_id.to_string(),
            device_id: device_id.to_string(),
            direction,
            bytes_done,
            bytes_total,
            last_published: None,
        }
    }

    /// Adds bytes and publishes at most once per interval; true when it published.
    pub fn add(&mut self, bytes: u64) -> bool {
        self.bytes_done += bytes;
        let due = self.last_published.map(|at| at.elapsed() >= PROGRESS_INTERVAL).unwrap_or(true);
        if due {
            self.publish();
        }
        due
    }

    /// Moves the position, e.g. back to what the peer really stored after a resume.
    pub fn seek(&mut self, bytes_done: u64) {
        self.bytes_done = bytes_done;
    }

    /// Publishes the current position regardless of throttling.
    pub fn flush(&mut self) {
        self.publish();
    }

    fn publish(&mut self) {
        self.last_published = Some(Instant::now());
        self.events.publish(NodeEvent::TransferProgress {
            transfer_id: self.transfer_id.clone(),
            device_id: self.device_id.clone(),
            direction: self.direction,
            bytes_done: self.bytes_done,
            bytes_total: self.bytes_total,
        });
    }
}
