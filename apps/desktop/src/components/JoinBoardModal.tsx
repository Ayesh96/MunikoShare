import { useState } from "react";

import {
  BOARD_CANCEL,
  BOARD_CODE_LABEL,
  BOARD_JOIN_CODE_PROMPT,
  BOARD_JOIN_CONFIRM,
  BOARD_JOIN_OPEN_PROMPT,
  boardJoinTitle,
} from "../messages/board";
import type { ObservedBoard } from "../types/board";
import { Modal, TextInput } from "./ui";

export interface JoinBoardModalProps {
  board: ObservedBoard;
  onJoin: (code?: string) => Promise<boolean>;
  onClose: () => void;
}

export function JoinBoardModal({ board, onJoin, onClose }: JoinBoardModalProps) {
  const [code, setCode] = useState("");
  const [busy, setBusy] = useState(false);
  const needsCode = board.info.mode === "protected";

  async function submit() {
    setBusy(true);
    const ok = await onJoin(needsCode ? code.trim() : undefined);
    if (ok) onClose();
    else setBusy(false);
  }

  return (
    <Modal
      open
      title={boardJoinTitle(board.info.name)}
      onClose={onClose}
      actions={[
        { label: BOARD_CANCEL, onClick: onClose },
        { label: BOARD_JOIN_CONFIRM, onClick: submit, variant: "primary", disabled: busy || (needsCode && code.trim() === "") },
      ]}
    >
      <p>{needsCode ? BOARD_JOIN_CODE_PROMPT : BOARD_JOIN_OPEN_PROMPT}</p>
      {needsCode && <TextInput value={code} onChange={setCode} label={BOARD_CODE_LABEL} autoFocus />}
    </Modal>
  );
}
