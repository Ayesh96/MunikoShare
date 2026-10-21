//! Forwards board changes to every connected phone, at most one per throttle window.

use async_trait::async_trait;
use tokio::sync::broadcast::error::{RecvError, TryRecvError};
use tokio::sync::broadcast::Receiver;

use super::service::MobileService;
use crate::constants::mobile::BOARD_EVENT_THROTTLE;
use crate::error::Result;
use crate::events::{EventBus, NodeEvent};
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::mobile as msg;
use crate::util::task::BackgroundTask;

pub struct MobileBoardBridge {
    service: MobileService,
    events: EventBus,
    task: Option<BackgroundTask>,
    state: LifecycleState,
}

impl MobileBoardBridge {
    pub fn new(service: MobileService, events: EventBus) -> Self {
        Self { service, events, task: None, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for MobileBoardBridge {
    async fn start(&mut self) -> Result<()> {
        let service = self.service.clone();
        let mut receiver = self.events.subscribe();
        self.task = Some(BackgroundTask::spawn(|cancel| async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    changed = next_change(&mut receiver) => {
                        if !changed {
                            break;
                        }
                        // Wait out the burst, then send one event for all of it.
                        tokio::time::sleep(BOARD_EVENT_THROTTLE).await;
                        drain(&mut receiver);
                        service.announce_board();
                    }
                }
            }
        }));
        self.state = LifecycleState::Running;
        tracing::info!("{}", msg::BOARD_BRIDGE_STARTED);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(mut task) = self.task.take() {
            task.stop().await;
        }
        self.state = LifecycleState::Stopped;
        tracing::info!("{}", msg::BOARD_BRIDGE_STOPPED);
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

// False once the bus is closed.
async fn next_change(receiver: &mut Receiver<NodeEvent>) -> bool {
    loop {
        match receiver.recv().await {
            Ok(NodeEvent::BoardChanged { .. }) | Err(RecvError::Lagged(_)) => return true,
            Ok(_) => {}
            Err(RecvError::Closed) => return false,
        }
    }
}

fn drain(receiver: &mut Receiver<NodeEvent>) {
    while !matches!(receiver.try_recv(), Err(TryRecvError::Empty | TryRecvError::Closed)) {}
}
