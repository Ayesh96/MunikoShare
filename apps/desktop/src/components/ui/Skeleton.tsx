import styles from "./Skeleton.module.css";

export interface SkeletonProps {
  rows?: number;
}

/** Placeholder rows shaped like a list row (avatar circle + two text bars). */
export function Skeleton({ rows = 3 }: SkeletonProps) {
  return (
    <div className={styles.list} aria-hidden>
      {Array.from({ length: rows }, (_, index) => (
        <div key={index} className={styles.row}>
          <span className={styles.circle} />
          <span className={styles.lines}>
            <span className={styles.bar} />
            <span className={`${styles.bar} ${styles.short}`} />
          </span>
        </div>
      ))}
    </div>
  );
}
