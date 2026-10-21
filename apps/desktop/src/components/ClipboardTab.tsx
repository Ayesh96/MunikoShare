import { useClipboardHistory } from "../hooks/useClipboardHistory";
import type { DeviceInfo } from "../types/device";
import type { Tone } from "../types/ui";
import { ClipboardHistoryCard } from "./ClipboardHistoryCard";
import { SendTextCard } from "./SendTextCard";

export interface ClipboardTabProps {
  pairedPeers: DeviceInfo[];
  nameOf: (deviceId: string) => string;
  show: (message: string, tone: Tone) => void;
  onSendText: (peer: DeviceInfo, text: string) => void;
}

/** Mounted only while its tab is visible, so the history polling stops when the tab changes. */
export function ClipboardTab({ pairedPeers, nameOf, show, onSendText }: ClipboardTabProps) {
  const history = useClipboardHistory(show);
  return (
    <>
      <SendTextCard pairedPeers={pairedPeers} onSend={onSendText} />
      <ClipboardHistoryCard
        items={history.items}
        nameOf={nameOf}
        onCopy={history.copy}
        onTogglePin={history.togglePin}
        onClear={history.clear}
      />
    </>
  );
}
