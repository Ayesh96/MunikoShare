import type { ReactNode } from "react";

import styles from "./List.module.css";

export function List({ children }: { children: ReactNode }) {
  return <ul className={styles.list}>{children}</ul>;
}

export interface ListItemProps {
  title: string;
  subtitle?: string;
  leading?: ReactNode;
  trailing?: ReactNode;
  /** Show the whole title on several lines instead of one truncated line. */
  wrap?: boolean;
}

export function ListItem({ title, subtitle, leading, trailing, wrap }: ListItemProps) {
  return (
    <li className={styles.item}>
      {leading && <span className={styles.leading}>{leading}</span>}
      <span className={styles.text}>
        <span className={wrap ? `${styles.title} ${styles.titleWrap}` : styles.title}>{title}</span>
        {subtitle && <span className={styles.subtitle}>{subtitle}</span>}
      </span>
      {trailing}
    </li>
  );
}
