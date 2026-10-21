import { useId } from "react";

import { controlClass, Field } from "./Field";

export interface TextAreaProps {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  rows?: number;
  placeholder?: string;
  error?: string;
}

const DEFAULT_ROWS = 4;

export function TextArea({ value, onChange, label, rows = DEFAULT_ROWS, placeholder, error }: TextAreaProps) {
  const id = useId();
  return (
    <Field id={id} label={label} error={error}>
      <textarea
        id={id}
        className={controlClass(error)}
        value={value}
        rows={rows}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? `${id}-error` : undefined}
      />
    </Field>
  );
}
