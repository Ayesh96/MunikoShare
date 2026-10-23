import type { ReactNode } from "react";
import { useEffect, useId, useRef } from "react";

import { Button, type ButtonVariant } from "./Button";
import styles from "./Modal.module.css";

export interface ModalAction {
  label: string;
  onClick: () => void;
  variant?: ButtonVariant;
  disabled?: boolean;
}

export interface ModalProps {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  actions?: ModalAction[];
}

const FOCUSABLE = 'button, [href], input, textarea, select, [tabindex]:not([tabindex="-1"])';

/** Keeps Tab inside the dialog so keyboard users cannot reach the page behind it. */
function trapFocus(event: KeyboardEvent, dialog: HTMLElement | null) {
  const items = Array.from(dialog?.querySelectorAll<HTMLElement>(FOCUSABLE) ?? []).filter((item) => !item.hasAttribute("disabled"));
  if (items.length === 0) return;
  const first = items[0];
  const last = items[items.length - 1];
  if (event.shiftKey && (document.activeElement === first || document.activeElement === dialog)) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}

export function Modal({ open, title, onClose, children, actions = [] }: ModalProps) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const titleId = useId();

  useEffect(() => {
    if (!open) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialogRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
      if (event.key === "Tab") trapFocus(event, dialogRef.current);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      opener?.focus();
    };
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
