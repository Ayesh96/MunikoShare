import { useState } from "react";

import { CLIPBOARD_SHARE_ICON } from "../constants/icons";
import { CLIPBOARD_SHARE_LABEL, CLIPBOARD_SHARE_PLACEHOLDER, CLIPBOARD_SHARE_TITLE } from "../messages/ui";
import styles from "./ShareTextCard.module.css";
import { Button, Card, TextArea } from "./ui";

export function ShareTextCard({ onShare }: { onShare: (text: string) => Promise<boolean> }) {
  const [text, setText] = useState("");
  const empty = text.trim() === "";

  async function share() {
    if (!empty && (await onShare(text))) setText("");
  }

  return (
    <Card title={CLIPBOARD_SHARE_TITLE}>
      <TextArea value={text} onChange={setText} placeholder={CLIPBOARD_SHARE_PLACEHOLDER} rows={3} onSubmit={share} />
      <div className={styles.buttons}>
        <Button icon={CLIPBOARD_SHARE_ICON} label={CLIPBOARD_SHARE_LABEL} disabled={empty} onClick={share} />
      </div>
    </Card>
  );
}
