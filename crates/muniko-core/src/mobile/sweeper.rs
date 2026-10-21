//! Periodically expires tokens, sessions and inbox items.

use async_trait::async_trait;

use super::service::MobileService;
use crate::constants::mobile::SWEEP_INTERVAL;
use crate::error::Result;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::mobile as msg;
use crate::util::task::BackgroundTask;

pub struct MobileSweeper {
    service: MobileService,
    task: Option<BackgroundTask>,
    state: LifecycleState,
}

impl MobileSweeper {
    pub fn new(service: MobileService) -> Self {
        Self { service, task: None, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for MobileSweeper {
    async fn start(&mut self) -> Result<()> {
        let service = self.service.clone();
        self.task = Some(BackgroundTask::spawn(|cancel| async move {
            let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = ticker.tick() => service.sweep(),
                }
            }
        }));
        self.state = LifecycleState::Running;
        tracing::info!("{}", msg::SWEEPER_STARTED);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(mut task) = self.task.take() {
            task.stop().await;
        }
        self.state = LifecycleState::Stopped;
        tracing::info!("{}", msg::SWEEPER_STOPPED);
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}
