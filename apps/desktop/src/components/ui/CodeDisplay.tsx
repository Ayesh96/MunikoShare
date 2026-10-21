import styles from "./CodeDisplay.module.css";

export interface CodeDisplayProps {
  code: string;
}

/** Large read-only code, e.g. a pairing code. */
export function CodeDisplay({ code }: CodeDisplayProps) {
  return <div className={styles.code}>{code}</div>;
}
