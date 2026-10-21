import type { ReactNode } from "react";

import styles from "./List.module.css";

export function List({ children }: { children: ReactNode }) {
  return <ul className={styles.list}>{children}</ul>;
}

export interface ListItemProps {
  title: string;
  subtitle?: string;
  /** Right-aligned content such as a button or status. */
  trailing?: ReactNode;
  /** Content shown below the row, e.g. a progress bar. */
  footer?: ReactNode;
}

export function ListItem({ title, subtitle, trailing, footer }: ListItemProps) {
  return (
    <li className={styles.item}>
      <div className={styles.row}>
        <div className={styles.text}>
          <span className={styles.title}>{title}</span>
          {subtitle && <span className={styles.subtitle}>{subtitle}</span>}
        </div>
        {trailing}
      </div>
      {footer}
    </li>
  );
}
