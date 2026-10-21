import { useState } from "react";

import { PHONE_CANCEL_LABEL, PHONE_TEXT_CONFIRM, PHONE_TEXT_PLACEHOLDER, PHONE_TEXT_TITLE } from "../messages/phone";
import { Modal, TextArea } from "./ui";

export interface SendPhoneTextModalProps {
  onSend: (text: string) => Promise<boolean>;
  onClose: () => void;
}

export function SendPhoneTextModal({ onSend, onClose }: SendPhoneTextModalProps) {
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit() {
    setBusy(true);
    if (await onSend(text)) onClose();
    else setBusy(false);
  }

  return (
    <Modal
      open
      title={PHONE_TEXT_TITLE}
      onClose={onClose}
      actions={[
        { label: PHONE_CANCEL_LABEL, onClick: onClose },
        { label: PHONE_TEXT_CONFIRM, onClick: submit, variant: "primary", disabled: busy || text.trim() === "" },
      ]}
    >
      <TextArea value={text} onChange={setText} placeholder={PHONE_TEXT_PLACEHOLDER} />
    </Modal>
  );
}
