import type { ReactNode } from "react";

import styles from "./Post.module.css";

export interface PostProps {
  meta: string;
  text: string;
  /** Content below the text, e.g. a copy button. */
  footer?: ReactNode;
}

export function Post({ meta, text, footer }: PostProps) {
  return (
    <li className={styles.post}>
      <span className={styles.meta}>{meta}</span>
      <p className={styles.body}>{text}</p>
      {footer}
    </li>
  );
}
