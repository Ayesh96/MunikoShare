import styles from "./Badge.module.css";

export interface BadgeProps {
  label: string;
  tone?: "neutral" | "success" | "danger";
}

export function Badge({ label, tone = "neutral" }: BadgeProps) {
  return <span className={`${styles.badge} ${styles[tone]}`}>{label}</span>;
}
