use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use super::context::BoardContext;
use super::membership::Membership;
use super::model::BoardMode;
use super::probe::probe_all;
use crate::constants::board::BOOTSTRAP_PROBE_INTERVAL;
use crate::error::Result;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::board as msg;
use crate::util::task::BackgroundTask;

/// On a device with no board: joins an open one found on the network, or creates one after the grace period.
pub struct BoardBootstrap {
    ctx: Arc<BoardContext>,
    task: Option<BackgroundTask>,
    state: LifecycleState,
}

impl BoardBootstrap {
    pub fn new(ctx: Arc<BoardContext>) -> Self {
        Self { ctx, task: None, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for BoardBootstrap {
    async fn start(&mut self) -> Result<()> {
        if self.task.is_none() {
            self.task = Some(BackgroundTask::spawn(|cancel| run(self.ctx.clone(), cancel)));
        }
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

async fn run(ctx: Arc<BoardContext>, cancel: CancellationToken) {
    let started = Instant::now();
    let membership = Membership::new(ctx.clone());
    while matches!(ctx.replica.record(), Ok(None)) {
        let found = probe_all(&ctx).await;
        if let Some((peer, _)) = found.iter().find(|(_, info)| info.board.mode == BoardMode::Open) {
            match membership.join(&peer.id, None).await {
                Ok(_) => return,
                Err(error) => tracing::debug!(%error, "{}", msg::LOG_BOOTSTRAP_FAILED),
            }
        }
        if !found.is_empty() && found.iter().all(|(_, info)| info.board.mode == BoardMode::Protected) {
            return;
        }
        if started.elapsed() >= ctx.config.discovery_grace && found.is_empty() {
            if let Err(error) = membership.create(&ctx.config.default_name, BoardMode::Open, None) {
                tracing::debug!(%error, "{}", msg::LOG_BOOTSTRAP_FAILED);
            }
            return;
        }
        tokio::select! {
            _ = cancel.cancelled() => return,
            _ = tokio::time::sleep(BOOTSTRAP_PROBE_INTERVAL) => {}
        }
    }
}
