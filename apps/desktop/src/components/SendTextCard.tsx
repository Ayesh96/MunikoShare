import { useState } from "react";

import {
  sendTextTo,
  SEND_TEXT_NO_PAIRED,
  SEND_TEXT_PLACEHOLDER,
  SEND_TEXT_TITLE,
} from "../messages/ui";
import type { DeviceInfo } from "../types/device";
import styles from "./SendTextCard.module.css";
import { Button, Card, TextArea } from "./ui";

export interface SendTextCardProps {
  pairedPeers: DeviceInfo[];
  onSend: (peer: DeviceInfo, text: string) => void;
}

export function SendTextCard({ pairedPeers, onSend }: SendTextCardProps) {
  const [text, setText] = useState("");

  return (
    <Card title={SEND_TEXT_TITLE}>
      <TextArea value={text} onChange={setText} placeholder={SEND_TEXT_PLACEHOLDER} />
      <div className={styles.buttons}>
        {pairedPeers.length === 0 && <span className={styles.hint}>{SEND_TEXT_NO_PAIRED}</span>}
        {pairedPeers.map((peer) => (
          <Button
            key={peer.id}
            size="sm"
            label={sendTextTo(peer.name)}
            disabled={text.trim() === ""}
            onClick={() => onSend(peer, text)}
          />
        ))}
      </div>
    </Card>
  );
}
