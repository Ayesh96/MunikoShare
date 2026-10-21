import styles from "./TextArea.module.css";

export interface TextAreaProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  readOnly?: boolean;
}

export function TextArea({ value, onChange, placeholder, readOnly }: TextAreaProps) {
  return (
    <textarea
      className={styles.area}
      value={value}
      placeholder={placeholder}
      readOnly={readOnly}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}
