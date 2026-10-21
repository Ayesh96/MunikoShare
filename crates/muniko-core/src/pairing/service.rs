use std::sync::Arc;
use std::time::Duration;

use super::client::PairingClient;
use crate::util::time::unix_now;
use super::code::PendingBook;
use super::profile::PairingProfile;
use super::routes::PairingRoutes;
use crate::constants::pairing::{CODE_TTL, MAX_ATTEMPTS};
use crate::device::DeviceInfo;
use crate::error::Result;
use crate::events::{EventBus, NodeEvent};
use crate::identity::Identity;
use crate::trust::{TrustStore, TrustedDevice};

/// Facade for pairing: initiate/confirm outgoing pairings, unpair, list trusted devices.
pub struct PairingService {
    client: PairingClient,
    store: Arc<dyn TrustStore>,
    events: EventBus,
    local: PairingProfile,
    book: Arc<PendingBook>,
}

impl PairingService {
    pub fn new(identity: Arc<Identity>, store: Arc<dyn TrustStore>, events: EventBus, local: PairingProfile) -> Self {
        Self::with_limits(identity, store, events, local, CODE_TTL, MAX_ATTEMPTS)
    }

    pub fn with_limits(
        identity: Arc<Identity>,
        store: Arc<dyn TrustStore>,
        events: EventBus,
        local: PairingProfile,
        ttl: Duration,
        max_attempts: u32,
    ) -> Self {
        Self { client: PairingClient::new(identity), store, events, local, book: Arc::new(PendingBook::new(ttl, max_attempts)) }
    }

    /// Server side: register with the HttpServer's route providers.
    pub fn routes(&self) -> PairingRoutes {
        PairingRoutes::new(self.book.clone(), self.store.clone(), self.events.clone(), self.local.clone())
    }

    /// Asks the peer to display a pairing code.
    pub async fn request(&self, peer: &DeviceInfo) -> Result<()> {
        self.client.request(peer, &self.local).await.inspect_err(|error| self.fail(peer, error))
    }

    /// Submits the code the user read on the peer; trusts the peer on success.
    pub async fn confirm(&self, peer: &DeviceInfo, code: &str) -> Result<()> {
        let remote = self.client.confirm(peer, code, &self.local).await.inspect_err(|error| self.fail(peer, error))?;
        let device = TrustedDevice { id: peer.id.clone(), name: remote.name.clone(), os: remote.os, paired_at: unix_now() };
        self.store.add(device).inspect_err(|error| self.fail(peer, error))?;
        self.events.publish(NodeEvent::PairingCompleted { device_id: peer.id.clone(), device_name: remote.name });
        Ok(())
    }

    pub fn unpair(&self, device_id: &str) -> Result<()> {
        self.store.remove(device_id)?;
        self.events.publish(NodeEvent::DeviceUnpaired { device_id: device_id.to_string() });
        Ok(())
    }

    pub fn trusted(&self) -> Vec<TrustedDevice> {
        self.store.list().unwrap_or_default()
    }

    fn fail(&self, peer: &DeviceInfo, error: &crate::error::MunikoError) {
        self.events.publish(NodeEvent::PairingFailed { device_id: peer.id.clone(), reason: error.to_string() });
    }
}
