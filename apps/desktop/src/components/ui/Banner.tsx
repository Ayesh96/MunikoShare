import { DISMISS_LABEL } from "../../messages/ui";
import type { Tone } from "../../types/ui";
import { Button } from "./Button";
import { Icon } from "./Icon";
import styles from "./Banner.module.css";

export interface BannerProps {
  message: string;
  tone?: Tone;
  actionLabel?: string;
  onAction?: () => void;
  onDismiss?: () => void;
}

export function Banner({ message, tone = "info", actionLabel, onAction, onDismiss }: BannerProps) {
  return (
    <div className={`${styles.banner} ${styles[tone]}`} role={tone === "danger" ? "alert" : "status"}>
      <span className={styles.message}>{message}</span>
      {actionLabel && onAction && <Button label={actionLabel} onClick={onAction} variant="secondary" size="sm" />}
      {onDismiss && <button type="button" className={styles.dismiss} onClick={onDismiss} aria-label={DISMISS_LABEL}><Icon name="close" size={16} /></button>}
    </div>
  );
}
