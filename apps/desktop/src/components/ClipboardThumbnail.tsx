import { useEffect, useState } from "react";

import { clipboardImage } from "../lib/tauriApi";
import { IMAGE_ALT } from "../messages/ui";
import styles from "./ClipboardThumbnail.module.css";

export interface ClipboardThumbnailProps {
  id: string;
  width: number;
  height: number;
}

/** Loads the image data only when the row is rendered; width and height reserve the space. */
export function ClipboardThumbnail({ id, width, height }: ClipboardThumbnailProps) {
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

  return <img className={styles.image} src={src ?? undefined} width={width} height={height} alt={IMAGE_ALT} />;
}
