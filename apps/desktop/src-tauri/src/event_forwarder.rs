//! Forwards core NodeEvents to the frontend until stopped.

use async_trait::async_trait;
use muniko_core::{Lifecycle, LifecycleState, NodeEvent, Result};
use tauri::async_runtime::{spawn, JoinHandle};
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, oneshot};
use tokio::sync::broadcast::error::RecvError;

use crate::constants::app::NODE_EVENT_NAME;
use crate::messages::{errors, logs};

pub struct EventForwarder {
    app: AppHandle,
    events: Option<broadcast::Receiver<NodeEvent>>,
    stop_signal: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
    state: LifecycleState,
}

impl EventForwarder {
    pub fn new(app: AppHandle, events: broadcast::Receiver<NodeEvent>) -> Self {
        Self { app, events: Some(events), stop_signal: None, task: None, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for EventForwarder {
    async fn start(&mut self) -> Result<()> {
        let Some(events) = self.events.take() else { return Ok(()) };
        let (stop_signal, stopped) = oneshot::channel();
        self.stop_signal = Some(stop_signal);
        self.task = Some(spawn(forward(self.app.clone(), events, stopped)));
        self.state = LifecycleState::Running;
        tracing::info!("{}", logs::FORWARDER_STARTED);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(stop_signal) = self.stop_signal.take() {
            let _ = stop_signal.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
        self.state = LifecycleState::Stopped;
        tracing::info!("{}", logs::FORWARDER_STOPPED);
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

async fn forward(app: AppHandle, mut events: broadcast::Receiver<NodeEvent>, mut stopped: oneshot::Receiver<()>) {
    loop {
        tokio::select! {
            _ = &mut stopped => break,
            received = events.recv() => match received {
                Ok(event) => emit(&app, &event),
                Err(RecvError::Lagged(_)) => tracing::warn!("{}", logs::EVENTS_LAGGED),
                Err(RecvError::Closed) => break,
            },
        }
    }
}

fn emit(app: &AppHandle, event: &NodeEvent) {
    if let Err(error) = app.emit(NODE_EVENT_NAME, event) {
        tracing::warn!("{}: {}", errors::EMIT_FAILED, error);
    }
}
