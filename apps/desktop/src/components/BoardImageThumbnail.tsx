import { useEffect, useState } from "react";

import { boardImage } from "../lib/boardApi";
import { BOARD_IMAGE_ALT } from "../messages/board";
import styles from "./ClipboardThumbnail.module.css";

/** Loads the image only when the row is rendered; a failed load leaves the placeholder. */
export function BoardImageThumbnail({ itemId }: { itemId: string }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    boardImage(itemId)
      .then((url) => !cancelled && setSrc(url))
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [itemId]);

  return <img className={styles.board} src={src ?? undefined} alt={BOARD_IMAGE_ALT} />;
}
