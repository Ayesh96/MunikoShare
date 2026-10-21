//! Finds other Muniko devices on the LAN via mDNS, with a UDP broadcast fallback and manual peers.

mod announcement;
mod broadcast;
mod registry;
mod txt;
mod udp;

#[cfg(test)]
mod tests;

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use mdns_sd::{ResolvedService, ServiceDaemon, ServiceEvent, ServiceInfo};

use crate::constants::discovery::{HOST_NAME_SUFFIX, INSTANCE_NAME_LEN, SERVICE_TYPE, SHUTDOWN_TIMEOUT_MS};
use crate::device::DeviceInfo;
use crate::error::{MunikoError, Result};
use crate::events::EventBus;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::discovery as messages;
use crate::net::candidate_addresses;
use broadcast::{BroadcastConfig, BroadcastService};
use registry::PeerRegistry;

use crate::util::peers::PeerDirectory;
use crate::util::task::BackgroundTask;

pub struct DiscoveryService {
    local: DeviceInfo,
    state: LifecycleState,
    registry: Arc<PeerRegistry>,
    broadcast: BroadcastService,
    running: Option<Running>,
}

/// Handles that exist only while discovery is running.
struct Running {
    daemon: ServiceDaemon,
    local_fullname: String,
    task: BackgroundTask,
}

fn discovery_error(error: impl std::fmt::Display) -> MunikoError {
    MunikoError::Discovery(error.to_string())
}

impl DiscoveryService {
    pub fn new(local: DeviceInfo, events: EventBus) -> Self {
        Self::with_broadcast(local, events, BroadcastConfig::default())
    }

    fn with_broadcast(local: DeviceInfo, events: EventBus, config: BroadcastConfig) -> Self {
        let registry = Arc::new(PeerRegistry::new(local.id.clone(), events.clone()));
        let broadcast = BroadcastService::new(local.clone(), registry.clone(), config);
        Self { local, state: LifecycleState::Created, registry, broadcast, running: None }
    }

    /// Currently known peers, excluding the local device.
    pub fn peers(&self) -> Vec<DeviceInfo> {
        self.registry.snapshot()
    }

    /// Adds a peer found by IP; it stays until `remove_manual_peer` or stop, and publishes `PeerAdded`.
    /// A live view of this service's peers for other features.
    pub fn directory(&self) -> Arc<dyn PeerDirectory> {
        self.registry.clone()
    }

    pub fn add_manual_peer(&self, device: DeviceInfo) {
        self.registry.add_manual(device);
    }

    pub fn remove_manual_peer(&self, device_id: &str) {
        self.registry.remove_manual(device_id);
    }

    fn local_service_info(&self) -> Result<ServiceInfo> {
        let short_id: String = self.local.id.chars().take(INSTANCE_NAME_LEN).collect();
        let host_name = format!("{short_id}{HOST_NAME_SUFFIX}");
        let info = ServiceInfo::new(SERVICE_TYPE, &short_id, &host_name, "", self.local.port, txt::encode(&self.local))
            .map_err(discovery_error)?;
        Ok(info.enable_addr_auto())
    }
}

#[async_trait]
impl Lifecycle for DiscoveryService {
    async fn start(&mut self) -> Result<()> {
        if self.state == LifecycleState::Running {
            return Err(MunikoError::InvalidState(format!("{:?}", self.state)));
        }
        let daemon = ServiceDaemon::new().map_err(discovery_error)?;
        let info = self.local_service_info()?;
        let local_fullname = info.get_fullname().to_string();
        daemon.register(info).map_err(discovery_error)?;
        let receiver = daemon.browse(SERVICE_TYPE).map_err(discovery_error)?;

        let handler = EventHandler { registry: self.registry.clone() };
        let task = BackgroundTask::spawn(|cancel| async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    event = receiver.recv_async() => match event {
                        Ok(event) => handler.handle(event),
                        Err(_) => break,
                    },
                }
            }
            tracing::debug!("{}", messages::EVENT_CHANNEL_CLOSED);
        });

        self.running = Some(Running { daemon, local_fullname, task });
        // The fallback is optional: without it mDNS still works.
        if let Err(error) = self.broadcast.start().await {
            tracing::warn!("{}: {error}", messages::BROADCAST_UNAVAILABLE);
        }
        self.state = LifecycleState::Running;
        tracing::info!("{}", messages::STARTED);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.broadcast.stop().await?;
        if let Some(mut running) = self.running.take() {
            // Best effort: the daemon is shut down right after either way.
            let _ = running.daemon.unregister(&running.local_fullname);
            let _ = running.daemon.stop_browse(SERVICE_TYPE);
            confirm_shutdown(&running.daemon).await;
            running.task.stop().await;
        }
        self.registry.clear();
        self.state = LifecycleState::Stopped;
        tracing::info!("{}", messages::STOPPED);
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

async fn confirm_shutdown(daemon: &ServiceDaemon) {
    let confirmed = match daemon.shutdown() {
        Ok(receiver) => tokio::time::timeout(Duration::from_millis(SHUTDOWN_TIMEOUT_MS), receiver.recv_async()).await.is_ok(),
        Err(_) => false,
    };
    if !confirmed {
        tracing::warn!("{}", messages::SHUTDOWN_UNCONFIRMED);
    }
}

/// Turns mDNS events into registry updates.
struct EventHandler {
    registry: Arc<PeerRegistry>,
}

impl EventHandler {
    fn handle(&self, event: ServiceEvent) {
        match event {
            ServiceEvent::ServiceResolved(service) => self.on_resolved(*service),
            ServiceEvent::ServiceRemoved(_, fullname) => self.registry.mdns_removed(&fullname),
            _ => {}
        }
    }

    fn on_resolved(&self, service: ResolvedService) {
        let resolved: Vec<IpAddr> = service.addresses.iter().map(|address| address.to_ip_addr()).collect();
        let records = service.txt_properties.clone().into_property_map_str();
        match txt::decode(&records, candidate_addresses(&resolved), service.port) {
            Some(device) => self.registry.observe_mdns(service.fullname, device),
            None => tracing::debug!("{}", messages::PEER_INVALID),
        }
    }
}
