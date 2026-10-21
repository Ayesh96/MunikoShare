use async_trait::async_trait;

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Created,
    Running,
    Stopped,
}

/// Every long-running service implements this so the node can start and stop it in order.
#[async_trait]
pub trait Lifecycle: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn state(&self) -> LifecycleState;
}
