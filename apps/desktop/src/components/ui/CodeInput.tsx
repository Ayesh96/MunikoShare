import { TextInput } from "./TextInput";

export interface CodeInputProps {
  value: string;
  onChange: (value: string) => void;
  length?: number;
  label?: string;
  error?: string;
  autoFocus?: boolean;
}

const DEFAULT_CODE_LENGTH = 6;

export function CodeInput({ value, onChange, length = DEFAULT_CODE_LENGTH, label, error, autoFocus }: CodeInputProps) {
  const handleChange = (raw: string) => onChange(raw.replace(/\D/g, "").slice(0, length));
  return (
    <TextInput
      variant="code"
      value={value}
      onChange={handleChange}
      label={label}
      error={error}
      maxLength={length}
      inputMode="numeric"
      autoFocus={autoFocus}
    />
  );
}
