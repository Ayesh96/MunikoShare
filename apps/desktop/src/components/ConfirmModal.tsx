import { BOARD_CANCEL, BOARD_CONFIRM_LABEL } from "../messages/board";
import { Modal } from "./ui";

export interface ConfirmModalProps {
  title: string;
  message: string;
  confirmLabel?: string;
  onConfirm: () => void;
  onClose: () => void;
}

export function ConfirmModal({ title, message, confirmLabel = BOARD_CONFIRM_LABEL, onConfirm, onClose }: ConfirmModalProps) {
  return (
    <Modal
      open
      title={title}
      onClose={onClose}
      actions={[
        { label: BOARD_CANCEL, onClick: onClose },
        { label: confirmLabel, onClick: onConfirm, variant: "primary" },
      ]}
    >
      <p>{message}</p>
    </Modal>
  );
}
