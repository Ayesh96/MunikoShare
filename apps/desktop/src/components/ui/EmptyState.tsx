import { EMPTY_ICON_SIZE, type IconName } from "../../constants/icons";
import { Button } from "./Button";
import styles from "./EmptyState.module.css";
import { Icon } from "./Icon";

export interface EmptyStateProps {
  title: string;
  hint?: string;
  icon?: IconName;
  actionLabel?: string;
  onAction?: () => void;
}

export function EmptyState({ title, hint, icon, actionLabel, onAction }: EmptyStateProps) {
  return (
    <div className={styles.empty}>
      {icon && <div className={styles.icon}><Icon name={icon} size={EMPTY_ICON_SIZE} /></div>}
      <h3 className={styles.title}>{title}</h3>
      {hint && <p className={styles.hint}>{hint}</p>}
      {actionLabel && onAction && <div className={styles.action}><Button label={actionLabel} onClick={onAction} variant="secondary" /></div>}
    </div>
  );
}
