import type { HTMLAttributes } from "react";
import { useId } from "react";

import { controlClass, Field } from "./Field";
import styles from "./TextInput.module.css";

export interface TextInputProps {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  placeholder?: string;
  error?: string;
  maxLength?: number;
  inputMode?: HTMLAttributes<HTMLInputElement>["inputMode"];
  autoFocus?: boolean;
  variant?: "default" | "code";
}

export function TextInput({ value, onChange, label, placeholder, error, maxLength, inputMode, autoFocus, variant = "default" }: TextInputProps) {
  const id = useId();
  return (
    <Field id={id} label={label} error={error}>
      <input
        id={id}
        className={controlClass(error, variant === "code" ? styles.code : "")}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        maxLength={maxLength}
        inputMode={inputMode}
        autoFocus={autoFocus}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? `${id}-error` : undefined}
      />
    </Field>
  );
}
