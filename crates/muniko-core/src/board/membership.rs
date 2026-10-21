use std::sync::Arc;

use super::context::BoardContext;
use super::model::{Board, BoardMode, BoardRecord, ItemKind, JoinSecret};
use super::wire::JoinRequest;
use crate::constants::board::{MAX_BOARD_NAME_CHARS, MAX_MEMBER_NAME_CHARS, MIN_JOIN_CODE_CHARS};
use crate::board::BoardError;
use crate::error::{MunikoError, Result};
use crate::messages::board as msg;
use crate::util::time::unix_now;

/// Creating, joining and leaving the board.
pub struct Membership {
    ctx: Arc<BoardContext>,
}

impl Membership {
    pub fn new(ctx: Arc<BoardContext>) -> Self {
        Self { ctx }
    }

    pub fn create(&self, name: &str, mode: BoardMode, join_code: Option<&str>) -> Result<Board> {
        self.ensure_no_board()?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > MAX_BOARD_NAME_CHARS {
            return Err(MunikoError::from(BoardError::InvalidName));
        }
        let join_secret = match mode {
            BoardMode::Open => None,
            BoardMode::Protected => Some(JoinSecret::create(checked_code(join_code)?)),
        };
        let board = Board { id: uuid::Uuid::new_v4().to_string(), name: name.to_string(), mode, created_at: unix_now() };
        self.ctx.replica.store().save_board(&BoardRecord { board: board.clone(), join_secret })?;
        self.add_self(&board.id)?;
        tracing::info!(board = %board.id, "{}", msg::LOG_CREATED);
        self.ctx.publish_changed(&board.id);
        Ok(board)
    }

    pub async fn join(&self, peer_id: &str, code: Option<&str>) -> Result<Board> {
        self.ensure_no_board()?;
        let peer = self.ctx.find_peer(peer_id)?;
        let info = self.ctx.client.info(&peer).await?.ok_or_else(|| MunikoError::from(BoardError::NoBoard))?;
        let member = self.ctx.signer.sign(&info.board.id, self.member_kind())?;
        let request = JoinRequest { board_id: info.board.id.clone(), code: code.map(str::to_string), member: member.clone() };
        let response = self.ctx.client.join(&peer, &request).await?;
        self.ensure_no_board()?;
        if response.board.board.id != info.board.id {
            return Err(MunikoError::from(BoardError::BoardMismatch));
        }
        self.ctx.replica.store().save_board(&response.board)?;
        let mut items = response.items;
        items.push(member.clone());
        self.ctx.replica.accept(items, unix_now())?;
        self.ctx.outbox.push(member);
        tracing::info!(board = %info.board.id, "{}", msg::LOG_JOINED);
        self.ctx.publish_changed(&info.board.id);
        Ok(response.board.board)
    }

    pub fn leave(&self) -> Result<()> {
        let record = self.ctx.replica.require_record()?;
        self.ctx.replica.store().clear()?;
        tracing::info!(board = %record.board.id, "{}", msg::LOG_LEFT);
        self.ctx.publish_changed(&record.board.id);
        Ok(())
    }

    fn ensure_no_board(&self) -> Result<()> {
        match self.ctx.replica.record()? {
            Some(_) => Err(MunikoError::from(BoardError::AlreadyOnBoard)),
            None => Ok(()),
        }
    }

    fn member_kind(&self) -> ItemKind {
        ItemKind::Member { device_id: self.ctx.local_id().to_string(), name: self.ctx.device_name.chars().take(MAX_MEMBER_NAME_CHARS).collect(), cert_der: self.ctx.signer.cert_der().to_vec() }
    }

    fn add_self(&self, board_id: &str) -> Result<()> {
        let member = self.ctx.signer.sign(board_id, self.member_kind())?;
        let accepted = self.ctx.replica.accept(vec![member], unix_now())?;
        if accepted.rejected > 0 {
            return Err(MunikoError::from(BoardError::ItemRejected));
        }
        Ok(())
    }
}

fn checked_code(code: Option<&str>) -> Result<&str> {
    let code = code.map(str::trim).filter(|code| !code.is_empty()).ok_or_else(|| MunikoError::from(BoardError::CodeRequired))?;
    if code.chars().count() < MIN_JOIN_CODE_CHARS {
        return Err(MunikoError::from(BoardError::CodeTooShort));
    }
    Ok(code)
}
