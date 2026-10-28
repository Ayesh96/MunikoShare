import { useState } from "react";

import { CLIPBOARD_SHARE_ICON } from "../constants/icons";
import { CLIPBOARD_SHARE_LABEL, CLIPBOARD_SHARE_PLACEHOLDER, CLIPBOARD_SHARE_TITLE } from "../messages/ui";
import styles from "./ShareCard.module.css";
import { blobKey } from "../lib/blobKey";
import { StagedImage } from "./StagedImage";
import { Button, Card, TextArea } from "./ui";

export interface ShareCardProps {
  images: Blob[];
  onRemoveImage: (image: Blob) => void;
  onShareImage: (image: Blob) => Promise<boolean>;
  onShareText: (text: string) => Promise<boolean>;
}

/** Draft area: pasted images and typed text are shared only when Share is pressed. */
export function ShareCard({ images, onRemoveImage, onShareImage, onShareText }: ShareCardProps) {
  const [text, setText] = useState("");
  const hasText = text.trim() !== "";
  const empty = images.length === 0 && !hasText;

  async function share() {
    for (const image of images) {
      if (!(await onShareImage(image))) return;
      onRemoveImage(image);
    }
    if (hasText && (await onShareText(text))) setText("");
  }

  return (
    <Card title={CLIPBOARD_SHARE_TITLE}>
      <TextArea value={text} onChange={setText} placeholder={CLIPBOARD_SHARE_PLACEHOLDER} rows={3} onSubmit={share} />
      {images.length > 0 && (
        <div className={styles.grid}>
          {images.map((image) => (
            <StagedImage key={blobKey(image)} image={image} onRemove={() => onRemoveImage(image)} />
          ))}
        </div>
      )}
      <div className={styles.buttons}>
        <Button icon={CLIPBOARD_SHARE_ICON} label={CLIPBOARD_SHARE_LABEL} disabled={empty} onClick={share} />
      </div>
    </Card>
  );
}
