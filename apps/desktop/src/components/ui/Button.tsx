import type { IconName } from "../../constants/icons";
import styles from "./Button.module.css";
import { Icon } from "./Icon";

export type ButtonVariant = "primary" | "secondary" | "tertiary" | "destructive";

export interface ButtonProps {
  label: string;
  onClick: () => void;
  icon?: IconName;
  variant?: ButtonVariant;
  size?: "sm" | "md";
  disabled?: boolean;
  title?: string;
}

export function Button({ label, onClick, icon, variant = "primary", size = "md", disabled, title }: ButtonProps) {
  return (
    <button
      type="button"
      className={`${styles.button} ${styles[variant]} ${styles[size]}`}
      onClick={onClick}
      disabled={disabled}
      title={title}
    >
      {icon && <Icon name={icon} />}
      {label}
    </button>
  );
}
