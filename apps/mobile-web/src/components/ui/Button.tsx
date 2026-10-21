import type { ReactNode } from "react";

import styles from "./Button.module.css";

export interface ButtonProps {
  label: string;
  onClick?: () => void;
  /** Renders a download link instead of a button; disabled shows a plain disabled button. */
  href?: string;
  icon?: ReactNode;
  variant?: "primary" | "secondary";
  size?: "sm" | "md";
  disabled?: boolean;
  fullWidth?: boolean;
}

export function Button({ label, onClick, href, icon, variant = "primary", size = "md", disabled, fullWidth }: ButtonProps) {
  const className = `${styles.button} ${styles[variant]} ${styles[size]} ${fullWidth ? styles.full : ""}`;
  const content = (
    <>
      {icon && <span aria-hidden>{icon}</span>}
      {label}
    </>
  );
  if (href && !disabled) return <a className={className} href={href} download>{content}</a>;
  return <button className={className} onClick={onClick} disabled={disabled}>{content}</button>;
}
