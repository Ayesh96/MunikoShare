import type { ReactNode } from "react";

import styles from "./Field.module.css";

export interface FieldProps {
  id: string;
  label?: string;
  error?: string;
  children: ReactNode;
}

/** Label + control + error layout shared by the text inputs. */
export function Field({ id, label, error, children }: FieldProps) {
  return (
    <div className={styles.field}>
      {label && <label className={styles.label} htmlFor={id}>{label}</label>}
      {children}
      {error && <span id={`${id}-error`} className={styles.error} role="alert">{error}</span>}
    </div>
  );
}

export function controlClass(error?: string, extra = ""): string {
  return [styles.control, error ? styles.invalid : "", extra].filter(Boolean).join(" ");
}
