import type { Tone } from "../../types/api";
import styles from "./Banner.module.css";

export function Banner({ message, tone = "info" }: { message: string; tone?: Tone }) {
  return <p className={`${styles.banner} ${styles[tone]}`} role="status">{message}</p>;
}
