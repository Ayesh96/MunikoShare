import type { ReactNode } from "react";

import styles from "./Button.module.css";

export interface ButtonProps {
  label: string;
  onClick: () => void;
  icon?: ReactNode;
  variant?: "primary" | "secondary";
  size?: "sm" | "md";
  disabled?: boolean;
}

export function Button({ label, onClick, icon, variant = "primary", size = "md", disabled }: ButtonProps) {
  return (
    <button className={`${styles.button} ${styles[variant]} ${styles[size]}`} onClick={onClick} disabled={disabled}>
      {icon && <span aria-hidden>{icon}</span>}
      {label}
    </button>
  );
}
