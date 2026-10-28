import { useState } from "react";

import { CLIPBOARD_SHARE_ICON } from "../constants/icons";
import { useObjectUrl } from "../hooks/useObjectUrl";
import {
  CLIPBOARD_SHARE_IMAGE_ALT,
  CLIPBOARD_SHARE_LABEL,
  CLIPBOARD_SHARE_PLACEHOLDER,
  CLIPBOARD_SHARE_REMOVE_IMAGE_LABEL,
  CLIPBOARD_SHARE_TITLE,
} from "../messages/ui";
import styles from "./ShareCard.module.css";
import { Button, Card, TextArea } from "./ui";

export interface ShareCardProps {
  image: Blob | null;
  onRemoveImage: () => void;
  onShareImage: (image: Blob) => Promise<boolean>;
  onShareText: (text: string) => Promise<boolean>;
}

/** Draft area: the pasted image and typed text are shared only when Share is pressed. */
export function ShareCard({ image, onRemoveImage, onShareImage, onShareText }: ShareCardProps) {
  const [text, setText] = useState("");
  const previewUrl = useObjectUrl(image);
  const hasText = text.trim() !== "";
  const empty = !image && !hasText;

  async function share() {
    if (image) {
      if (!(await onShareImage(image))) return;
      onRemoveImage();
    }
    if (hasText && (await onShareText(text))) setText("");
  }

  return (
    <Card title={CLIPBOARD_SHARE_TITLE}>
      <TextArea value={text} onChange={setText} placeholder={CLIPBOARD_SHARE_PLACEHOLDER} rows={3} onSubmit={share} />
      {previewUrl && (
        <div className={styles.draftImage}>
          <img className={styles.preview} src={previewUrl} alt={CLIPBOARD_SHARE_IMAGE_ALT} />
          <Button variant="tertiary" size="sm" label={CLIPBOARD_SHARE_REMOVE_IMAGE_LABEL} onClick={onRemoveImage} />
        </div>
      )}
      <div className={styles.buttons}>
        <Button icon={CLIPBOARD_SHARE_ICON} label={CLIPBOARD_SHARE_LABEL} disabled={empty} onClick={share} />
      </div>
    </Card>
  );
}
