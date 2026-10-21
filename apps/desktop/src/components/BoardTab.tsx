import { useState } from "react";

import { useBoard } from "../hooks/useBoard";
import {
  BOARD_CONFLICT_ACTION,
  BOARD_LEAVE_LABEL,
  BOARD_LEAVE_TITLE,
  BOARD_RETRY_JOIN_LABEL,
  boardConflict,
  boardJoinFailed,
  boardLeavePrompt,
} from "../messages/board";
import type { ObservedBoard } from "../types/board";
import type { Tone } from "../types/ui";
import { BoardComposer } from "./BoardComposer";
import { BoardFeed } from "./BoardFeed";
import { BoardHeader } from "./BoardHeader";
import { BoardSetup } from "./BoardSetup";
import { ConfirmModal } from "./ConfirmModal";
import { CreateBoardModal } from "./CreateBoardModal";
import { JoinBoardModal } from "./JoinBoardModal";
import { Banner } from "./ui";

type Dialog = { type: "create" } | { type: "leave" } | { type: "join"; board: ObservedBoard; replace: boolean };

/** Mounted only while its tab is visible, so file drops post only from here. */
export function BoardTab({ show }: { show: (message: string, tone: Tone) => void }) {
  const board = useBoard(show);
  const [dialog, setDialog] = useState<Dialog | null>(null);
  const { state } = board;
  const close = () => setDialog(null);
  if (!state) return null;

  return (
    <>
      {board.failedJoin && !state.board && (
        <Banner
          tone="danger"
          message={boardJoinFailed(board.failedJoin.target.info.name)}
          actionLabel={BOARD_RETRY_JOIN_LABEL}
          onAction={board.retryJoin}
          onDismiss={board.dismissFailedJoin}
        />
      )}
      {state.conflicts.map((other) => (
        <Banner
          key={other.info.id}
          tone="danger"
          message={boardConflict(other.info.name, other.peerName)}
          actionLabel={BOARD_CONFLICT_ACTION}
          onAction={() => setDialog({ type: "join", board: other, replace: true })}
        />
      ))}
      {state.board ? (
        <>
          <BoardHeader board={state.board} keeper={state.keeper} onKeeperChange={board.setKeeper} onLeave={() => setDialog({ type: "leave" })} />
          <BoardComposer onPost={board.postText} onAddFiles={board.addFiles} />
          <BoardFeed items={state.items} onCopy={board.copy} onDownload={board.download} onDelete={board.remove} />
        </>
      ) : (
        <BoardSetup
          discovered={state.discoveredBoards}
          onCreate={() => setDialog({ type: "create" })}
          onJoin={(other) => setDialog({ type: "join", board: other, replace: false })}
        />
      )}
      {dialog?.type === "create" && <CreateBoardModal onCreate={board.create} onClose={close} />}
      {dialog?.type === "join" && (
        <JoinBoardModal board={dialog.board} onJoin={(code) => board.join(dialog.board, code, dialog.replace)} onClose={close} />
      )}
      {dialog?.type === "leave" && state.board && (
        <ConfirmModal
          title={BOARD_LEAVE_TITLE}
          message={boardLeavePrompt(state.board.name)}
          confirmLabel={BOARD_LEAVE_LABEL}
          onConfirm={() => {
            close();
            board.leave();
          }}
          onClose={close}
        />
      )}
    </>
  );
}
