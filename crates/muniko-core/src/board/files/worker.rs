//! Background caching: wakes on board changes and on a timer, tidies the cache, then fetches wanted files.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use super::auto::cache_wanted;
use super::availability::publish_if_changed;
use super::maintenance::reconcile;
use crate::board::context::BoardContext;
use crate::error::Result;
use crate::events::NodeEvent;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::board as msg;
use crate::util::task::BackgroundTask;

pub struct FileWorker {
    ctx: Arc<BoardContext>,
    task: Option<BackgroundTask>,
    state: LifecycleState,
}

impl FileWorker {
    pub fn new(ctx: Arc<BoardContext>) -> Self {
        Self { ctx, task: None, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for FileWorker {
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
    let mut events = ctx.events.subscribe();
    let mut ticker = interval(ctx.config.sync_interval);
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {}
            event = events.recv() => match event {
                Ok(NodeEvent::BoardChanged { .. }) | Err(RecvError::Lagged(_)) => {}
                Ok(_) => continue,
                Err(RecvError::Closed) => break,
            },
        }
        // A cancelled fetch keeps its .part, so the next start resumes it.
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = maintain(&ctx) => {}
        }
    }
}

async fn maintain(ctx: &BoardContext) {
    if let Err(error) = reconcile(ctx) {
        tracing::debug!(%error, "{}", msg::LOG_MAINTENANCE_FAILED);
    }
    if let Err(error) = cache_wanted(ctx).await {
        tracing::debug!(%error, "{}", msg::LOG_MAINTENANCE_FAILED);
    }
    publish_if_changed(ctx);
}
