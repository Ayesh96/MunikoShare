import { useEffect, useState } from "react";

import { clipboardImage } from "../lib/tauriApi";
import { IMAGE_ALT } from "../messages/ui";
import styles from "./ClipboardThumbnail.module.css";

export interface ClipboardThumbnailProps {
  id: string;
}

/** Loads the image data only when the row is rendered; it fills its parent tile. */
export function ClipboardThumbnail({ id }: ClipboardThumbnailProps) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    clipboardImage(id)
      .then((url) => !cancelled && setSrc(url))
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [id]);

  return <img className={styles.image} src={src ?? undefined} alt={IMAGE_ALT} />;
}
