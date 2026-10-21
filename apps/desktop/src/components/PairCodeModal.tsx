import { CLOSE_LABEL, PAIR_SHOW_TITLE, pairShowPrompt } from "../messages/ui";
import type { DeviceInfo } from "../types/device";
import { CodeDisplay, Modal } from "./ui";

export interface PairCodeModalProps {
  device: DeviceInfo;
  code: string;
  onClose: () => void;
}

/** Other side: shows the code the requesting device must type. */
export function PairCodeModal({ device, code, onClose }: PairCodeModalProps) {
  return (
    <Modal open title={PAIR_SHOW_TITLE} onClose={onClose} actions={[{ label: CLOSE_LABEL, onClick: onClose }]}>
      <p>{pairShowPrompt(device.name)}</p>
      <CodeDisplay code={code} />
    </Modal>
  );
}
