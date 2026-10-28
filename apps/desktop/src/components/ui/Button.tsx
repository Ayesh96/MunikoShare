import { ICON_COMPACT_SIZE, type IconName } from "../../constants/icons";
import styles from "./Button.module.css";
import { Icon } from "./Icon";

export type ButtonVariant = "primary" | "secondary" | "tertiary" | "destructive";

export interface ButtonProps {
  label: string;
  onClick: () => void;
  icon?: IconName;
  variant?: ButtonVariant;
  size?: "xs" | "sm" | "md";
  disabled?: boolean;
  title?: string;
  /** Show only the icon; the label becomes the accessible name and tooltip. */
  iconOnly?: boolean;
}

export function Button({ label, onClick, icon, variant = "primary", size = "md", disabled, title, iconOnly }: ButtonProps) {
  return (
    <button
      type="button"
      className={`${styles.button} ${styles[variant]} ${styles[size]}${iconOnly ? ` ${styles.iconOnly}` : ""}`}
      onClick={onClick}
      disabled={disabled}
      title={iconOnly ? label : title}
      aria-label={iconOnly ? label : undefined}
    >
      {icon && <Icon name={icon} size={size === "xs" ? ICON_COMPACT_SIZE : undefined} />}
      {!iconOnly && label}
    </button>
  );
}
