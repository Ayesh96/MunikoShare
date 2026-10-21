import { useState } from "react";

import {
  BOARD_CANCEL,
  BOARD_CODE_HINT,
  BOARD_CODE_LABEL,
  BOARD_CREATE_CONFIRM,
  BOARD_CREATE_TITLE,
  BOARD_NAME_LABEL,
  BOARD_NAME_PLACEHOLDER,
  BOARD_PROTECTED_SWITCH,
} from "../messages/board";
import type { BoardMode } from "../types/board";
import { Modal, Switch, TextInput } from "./ui";

export interface CreateBoardModalProps {
  onCreate: (name: string, mode: BoardMode, joinCode?: string) => Promise<boolean>;
  onClose: () => void;
}

export function CreateBoardModal({ onCreate, onClose }: CreateBoardModalProps) {
  const [name, setName] = useState("");
  const [protectedMode, setProtectedMode] = useState(false);
  const [code, setCode] = useState("");
  const [busy, setBusy] = useState(false);
  const valid = name.trim() !== "" && (!protectedMode || code.trim() !== "");

  async function submit() {
    setBusy(true);
    const ok = await onCreate(name.trim(), protectedMode ? "protected" : "open", protectedMode ? code.trim() : undefined);
    if (ok) onClose();
    else setBusy(false);
  }

  return (
    <Modal
      open
      title={BOARD_CREATE_TITLE}
      onClose={onClose}
      actions={[
        { label: BOARD_CANCEL, onClick: onClose },
        { label: BOARD_CREATE_CONFIRM, onClick: submit, variant: "primary", disabled: busy || !valid },
      ]}
    >
      <TextInput value={name} onChange={setName} label={BOARD_NAME_LABEL} placeholder={BOARD_NAME_PLACEHOLDER} autoFocus />
      <Switch checked={protectedMode} onChange={setProtectedMode} label={BOARD_PROTECTED_SWITCH} />
      {protectedMode && <TextInput value={code} onChange={setCode} label={BOARD_CODE_LABEL} placeholder={BOARD_CODE_HINT} />}
    </Modal>
  );
}
