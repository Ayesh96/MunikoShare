//! UDP broadcast fallback for networks that block multicast.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

use super::announcement::Announcement;
use super::registry::PeerRegistry;
use super::udp::bind_broadcast_socket;
use crate::constants::discovery::{BROADCAST_BUFFER_SIZE, BROADCAST_INTERVAL, BROADCAST_PORT, PEER_TTL};
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::discovery as messages;
use crate::util::task::BackgroundTask;

#[derive(Debug, Clone, Copy)]
pub struct BroadcastConfig {
    pub port: u16,
    pub interval: Duration,
    pub peer_ttl: Duration,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self { port: BROADCAST_PORT, interval: BROADCAST_INTERVAL, peer_ttl: PEER_TTL }
    }
}

/// Announces this device and listens for others' announcements.
pub struct BroadcastService {
    local: DeviceInfo,
    registry: Arc<PeerRegistry>,
    config: BroadcastConfig,
    state: LifecycleState,
    task: Option<BackgroundTask>,
}

impl BroadcastService {
    pub fn new(local: DeviceInfo, registry: Arc<PeerRegistry>, config: BroadcastConfig) -> Self {
        Self { local, registry, config, state: LifecycleState::Created, task: None }
    }
}

#[async_trait]
impl Lifecycle for BroadcastService {
    async fn start(&mut self) -> Result<()> {
        if self.state == LifecycleState::Running {
            return Err(MunikoError::InvalidState(format!("{:?}", self.state)));
        }
        let socket = bind_broadcast_socket(self.config.port)?;
        let payload = Announcement::from_device(&self.local)
            .encode()
            .ok_or_else(|| MunikoError::Discovery(messages::BROADCAST_ENCODE_FAILED.to_string()))?;
        let worker = Worker { socket, payload, local_id: self.local.id.clone(), registry: self.registry.clone(), config: self.config };
        self.task = Some(BackgroundTask::spawn(|cancel| worker.run(cancel)));
        self.state = LifecycleState::Running;
        tracing::info!("{}", messages::BROADCAST_STARTED);
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
    socket: UdpSocket,
    payload: Vec<u8>,
    local_id: String,
    registry: Arc<PeerRegistry>,
    config: BroadcastConfig,
}

impl Worker {
    async fn run(self, cancel: CancellationToken) {
        let target = SocketAddr::from((Ipv4Addr::BROADCAST, self.config.port));
        let mut ticker = tokio::time::interval(self.config.interval);
        let mut buffer = vec![0u8; BROADCAST_BUFFER_SIZE];
        loop {
            tokio::select! {
                _ = cancel.cancelled() => return,
                _ = ticker.tick() => {
                    self.announce(target).await;
                    self.registry.expire_broadcast(self.config.peer_ttl, Instant::now());
                }
                received = self.socket.recv_from(&mut buffer) => match received {
                    Ok((length, sender)) => self.receive(&buffer[..length], sender),
                    Err(error) => tracing::debug!("{}: {error}", messages::BROADCAST_RECV_FAILED),
                },
            }
        }
    }

    async fn announce(&self, target: SocketAddr) {
        // Sending fails while offline; the next tick simply tries again.
        if let Err(error) = self.socket.send_to(&self.payload, target).await {
            tracing::debug!("{}: {error}", messages::BROADCAST_SEND_FAILED);
        }
    }

    fn receive(&self, bytes: &[u8], sender: SocketAddr) {
        let Some(announcement) = Announcement::decode(bytes) else {
            tracing::debug!("{}", messages::BROADCAST_INVALID);
            return;
        };
        let device = announcement.into_device(sender.ip());
        if device.id != self.local_id {
            self.registry.observe_broadcast(device, Instant::now());
        }
    }
}
