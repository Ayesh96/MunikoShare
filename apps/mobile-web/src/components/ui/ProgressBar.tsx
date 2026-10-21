import type { Tone } from "../../types/api";
import styles from "./ProgressBar.module.css";

export interface ProgressBarProps {
  /** Fraction between 0 and 1; out-of-range values are clamped. */
  value: number;
  tone?: Tone;
}

export function ProgressBar({ value, tone = "info" }: ProgressBarProps) {
  const percent = Math.round(Math.min(1, Math.max(0, value)) * 100);
  return (
    <div className={styles.track} role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent}>
      <div className={`${styles.fill} ${styles[tone]}`} style={{ width: `${percent}%` }} />
    </div>
  );
}
