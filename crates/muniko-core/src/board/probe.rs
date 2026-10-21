use futures_util::future::join_all;

use super::context::BoardContext;
use super::model::BoardInfo;
use crate::device::DeviceInfo;
use crate::error::Result;
use crate::messages::board as msg;

/// Asks a peer about its board and remembers the answer.
pub async fn probe(ctx: &BoardContext, peer: &DeviceInfo) -> Result<Option<BoardInfo>> {
    let info = ctx.client.info(peer).await?;
    let local = ctx.replica.record()?.map(|record| record.board.id);
    let changed = ctx.observed.record(&peer.id, &peer.name, info.clone());
    let conflicting = matches!((&info, &local), (Some(seen), Some(mine)) if &seen.board.id != mine);
    if changed {
        ctx.publish_changed(&local.unwrap_or_default());
        if conflicting {
            tracing::warn!(peer = %peer.id, "{}", msg::LOG_CONFLICT);
        }
    }
    Ok(info)
}

/// Probes every known peer at once; unreachable peers are skipped.
pub async fn probe_all(ctx: &BoardContext) -> Vec<(DeviceInfo, BoardInfo)> {
    let peers = ctx.peers.peers();
    let ids: Vec<String> = peers.iter().map(|peer| peer.id.clone()).collect();
    let results = join_all(peers.iter().map(|peer| probe(ctx, peer))).await;
    if ctx.observed.forget_except(&ids) {
        ctx.publish_changed(&ctx.replica.record().ok().flatten().map(|r| r.board.id).unwrap_or_default());
    }
    peers.into_iter().zip(results).filter_map(|(peer, result)| Some((peer, result.ok()??))).collect()
}
