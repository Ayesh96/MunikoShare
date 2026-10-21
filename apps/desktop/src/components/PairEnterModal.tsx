import { useState } from "react";

import { PAIRING_CODE_LENGTH } from "../constants/app";
import { errorMessage } from "../lib/format";
import { PAIR_CANCEL_LABEL, PAIR_CONFIRM_LABEL, PAIR_ENTER_TITLE, pairEnterPrompt } from "../messages/ui";
import type { DeviceInfo } from "../types/device";
import { CodeInput, Modal } from "./ui";

export interface PairEnterModalProps {
  device: DeviceInfo;
  onConfirm: (code: string) => Promise<void>;
  onClose: () => void;
}

/** Initiator side: the user types the code shown on the other device. */
export function PairEnterModal({ device, onConfirm, onClose }: PairEnterModalProps) {
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  async function submit() {
    setBusy(true);
    setError(undefined);
    try {
      await onConfirm(code);
    } catch (failure) {
      setError(errorMessage(failure));
      setBusy(false);
    }
  }

  return (
    <Modal
      open
      title={PAIR_ENTER_TITLE}
      onClose={onClose}
      actions={[
        { label: PAIR_CANCEL_LABEL, onClick: onClose },
        { label: PAIR_CONFIRM_LABEL, onClick: submit, variant: "primary", disabled: busy || code.length !== PAIRING_CODE_LENGTH },
      ]}
    >
      <CodeInput value={code} onChange={setCode} label={pairEnterPrompt(device.name)} error={error} autoFocus />
    </Modal>
  );
}
