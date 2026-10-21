import {
  RELATIVE_TIME_MAX_DAYS,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from "../constants/app";

const relative = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

function relativeParts(elapsed: number): [number, Intl.RelativeTimeFormatUnit] {
  if (elapsed < SECONDS_PER_MINUTE) return [-elapsed, "second"];
  if (elapsed < SECONDS_PER_HOUR) return [-Math.floor(elapsed / SECONDS_PER_MINUTE), "minute"];
  if (elapsed < SECONDS_PER_DAY) return [-Math.floor(elapsed / SECONDS_PER_HOUR), "hour"];
  return [-Math.floor(elapsed / SECONDS_PER_DAY), "day"];
}

/** Relative text for recent unix-second timestamps, local date and time for older ones. */
export function formatTimestamp(unixSeconds: number, nowMs = Date.now()): string {
  const elapsed = Math.max(0, Math.floor(nowMs / 1000 - unixSeconds));
  if (elapsed >= RELATIVE_TIME_MAX_DAYS * SECONDS_PER_DAY) return new Date(unixSeconds * 1000).toLocaleString();
  const [value, unit] = relativeParts(elapsed);
  return relative.format(value, unit);
}
