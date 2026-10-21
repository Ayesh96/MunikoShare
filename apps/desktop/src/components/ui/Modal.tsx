import type { ReactNode } from "react";
import { useEffect, useId, useRef } from "react";

import { Button } from "./Button";
import styles from "./Modal.module.css";

export interface ModalAction {
  label: string;
  onClick: () => void;
  variant?: "primary" | "secondary";
  disabled?: boolean;
}

export interface ModalProps {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  actions?: ModalAction[];
}

export function Modal({ open, title, onClose, children, actions = [] }: ModalProps) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const titleId = useId();

  useEffect(() => {
    if (!open) return;
    dialogRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) return null;
  return (
    <div className={styles.backdrop} onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <div ref={dialogRef} className={styles.dialog} role="dialog" aria-modal="true" aria-labelledby={titleId} tabIndex={-1}>
        <h2 id={titleId} className={styles.title}>{title}</h2>
        <div>{children}</div>
        {actions.length > 0 && (
          <div className={styles.footer}>
            {actions.map((action) => (
              <Button key={action.label} label={action.label} onClick={action.onClick} variant={action.variant ?? "secondary"} disabled={action.disabled} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
