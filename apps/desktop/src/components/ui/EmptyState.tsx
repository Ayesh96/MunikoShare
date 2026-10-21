import type { ReactNode } from "react";

import styles from "./EmptyState.module.css";

export interface EmptyStateProps {
  title: string;
  hint?: string;
  icon?: ReactNode;
}

export function EmptyState({ title, hint, icon }: EmptyStateProps) {
  return (
    <div className={styles.empty}>
      {icon && <div className={styles.icon} aria-hidden>{icon}</div>}
      <div className={styles.title}>{title}</div>
      {hint && <div className={styles.hint}>{hint}</div>}
    </div>
  );
}
