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
  /** Extra content shown under the row, such as expanded options. */
  details?: ReactNode;
}

export function ListItem({ title, subtitle, leading, trailing, wrap, details }: ListItemProps) {
  return (
    <li className={styles.item}>
      <div className={styles.row}>
        {leading && <span className={styles.leading}>{leading}</span>}
        <span className={styles.text}>
          <span className={wrap ? `${styles.title} ${styles.titleWrap}` : styles.title}>{title}</span>
          {subtitle && <span className={styles.subtitle}>{subtitle}</span>}
        </span>
        {trailing && <span className={styles.trailing}>{trailing}</span>}
      </div>
      {details && <div className={styles.details}>{details}</div>}
    </li>
  );
}
