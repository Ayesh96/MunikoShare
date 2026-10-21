import type { Tone } from "../../types/ui";
import styles from "./ProgressBar.module.css";

export interface ProgressBarProps {
  /** Fraction between 0 and 1; out-of-range values are clamped. */
  value: number;
  label?: string;
  tone?: Tone;
}

export function ProgressBar({ value, label, tone = "info" }: ProgressBarProps) {
  const percent = Math.round(Math.min(1, Math.max(0, value)) * 100);
  return (
    <div className={styles.wrap}>
      {label && <span className={styles.label}>{label}</span>}
      <div className={styles.track} role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent} aria-label={label}>
        <div className={`${styles.fill} ${styles[tone]}`} style={{ width: `${percent}%` }} />
      </div>
    </div>
  );
}
