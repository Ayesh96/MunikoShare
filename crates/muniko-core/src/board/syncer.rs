use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::join_all;
use tokio::time::{interval, timeout};
use tokio_util::sync::CancellationToken;

use super::context::BoardContext;
use super::files::publish_if_changed;
use super::probe::probe;
use crate::constants::board::*;
use crate::device::DeviceInfo;
use crate::error::Result;
use crate::events::NodeEvent;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::messages::board as msg;
use crate::util::task::BackgroundTask;
use crate::util::time::unix_now;

/// Keeps the replica in step with member peers: immediate pushes plus periodic digest pulls.
pub struct BoardSyncer {
    ctx: Arc<BoardContext>,
    task: Option<BackgroundTask>,
    state: LifecycleState,
}

impl BoardSyncer {
    pub fn new(ctx: Arc<BoardContext>) -> Self {
        Self { ctx, task: None, state: LifecycleState::Created }
    }
}

#[async_trait]
impl Lifecycle for BoardSyncer {
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
            _ = ticker.tick() => sync_all(&ctx).await,
            _ = ctx.outbox.wait() => push_pending(&ctx).await,
            event = events.recv() => {
                if let Ok(NodeEvent::PeerFound { device } | NodeEvent::PeerAdded { device }) = event {
                    sync_peer_bounded(&ctx, &device).await;
                }
            }
        }
    }
}

async fn sync_all(ctx: &BoardContext) {
    let peers = ctx.peers.peers();
    join_all(peers.iter().map(|peer| sync_peer_bounded(ctx, peer))).await;
    match ctx.replica.prune(unix_now()) {
        Ok(0) | Err(_) => {}
        Ok(count) => {
            tracing::debug!(count, "{}", msg::LOG_PRUNED);
            ctx.publish_changed(&ctx.replica.record().ok().flatten().map(|r| r.board.id).unwrap_or_default());
        }
    }
    // Peers may have gone offline since the last pass.
    publish_if_changed(ctx);
}

async fn sync_peer_bounded(ctx: &BoardContext, peer: &DeviceInfo) {
    match timeout(PEER_SYNC_TIMEOUT, sync_peer(ctx, peer)).await {
        Ok(Ok(_)) => {}
        Ok(Err(error)) => tracing::debug!(peer = %peer.id, %error, "{}", msg::LOG_SYNC_FAILED),
        Err(_) => tracing::debug!(peer = %peer.id, "{}", msg::LOG_SYNC_FAILED),
    }
}

/// Checks the peer's board, then pulls what is missing; returns how many items were added.
async fn sync_peer(ctx: &BoardContext, peer: &DeviceInfo) -> Result<usize> {
    let info = probe(ctx, peer).await?;
    let Some(local) = ctx.replica.record()? else { return Ok(0) };
    match info {
        Some(info) if info.board.id == local.board.id => pull(ctx, peer, &local.board.id).await,
        _ => Ok(0),
    }
}

async fn pull(ctx: &BoardContext, peer: &DeviceInfo, board_id: &str) -> Result<usize> {
    let digest = ctx.client.digest(peer).await?;
    if digest.board_id != board_id {
        return Ok(0);
    }
    if ctx.files.holders.update(&peer.id, &digest.have) {
        publish_if_changed(ctx);
    }
    let now = unix_now();
    let wanted = ctx.replica.missing(&digest.entries, now)?;
    let mut fetched = Vec::new();
    for chunk in wanted.chunks(MAX_FETCH_IDS).take(MAX_PULL_ITEMS / MAX_FETCH_IDS) {
        fetched.extend(ctx.client.fetch(peer, chunk.to_vec()).await?);
    }
    if fetched.is_empty() {
        return Ok(0);
    }
    let accepted = ctx.replica.accept(fetched, now)?;
    if accepted.rejected > 0 {
        tracing::warn!(peer = %peer.id, count = accepted.rejected, "{}", msg::LOG_REJECTED_ITEMS);
    }
    if !accepted.added.is_empty() {
        ctx.publish_changed(board_id);
    }
    Ok(accepted.added.len())
}

async fn push_pending(ctx: &BoardContext) {
    let items = ctx.outbox.take();
    if items.is_empty() {
        return;
    }
    let peers = ctx.peers.peers();
    join_all(peers.iter().map(|peer| push_to(ctx, peer, &items))).await;
}

async fn push_to(ctx: &BoardContext, peer: &DeviceInfo, items: &[super::model::BoardItem]) {
    for chunk in items.chunks(MAX_PUSH_ITEMS) {
        let sent = timeout(PEER_SYNC_TIMEOUT, ctx.client.push(peer, chunk.to_vec())).await;
        if !matches!(sent, Ok(Ok(_))) {
            tracing::debug!(peer = %peer.id, "{}", msg::LOG_PUSH_FAILED);
            return;
        }
    }
}
