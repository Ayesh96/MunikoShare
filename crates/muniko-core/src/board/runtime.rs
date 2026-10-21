use async_trait::async_trait;

use super::bootstrap::BoardBootstrap;
use super::files::FileWorker;
use super::syncer::BoardSyncer;
use crate::error::Result;
use crate::lifecycle::{Lifecycle, LifecycleState};

/// The board's background work as one service: syncer, file worker, then bootstrap.
pub struct BoardRuntime {
    syncer: BoardSyncer,
    files: FileWorker,
    bootstrap: BoardBootstrap,
    state: LifecycleState,
}

impl BoardRuntime {
    pub fn new(syncer: BoardSyncer, files: FileWorker, bootstrap: BoardBootstrap) -> Self {
        Self { syncer, files, bootstrap, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for BoardRuntime {
    async fn start(&mut self) -> Result<()> {
        self.syncer.start().await?;
        self.files.start().await?;
        self.bootstrap.start().await?;
        self.state = LifecycleState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.bootstrap.stop().await?;
        self.files.stop().await?;
        self.syncer.stop().await?;
        self.state = LifecycleState::Stopped;
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}
