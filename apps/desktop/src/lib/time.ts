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

/** Local date and time that is safe inside a file name on every OS, e.g. 2026-10-28 14.03.07. */
export function fileNameStamp(date = new Date()): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}.${pad(date.getMinutes())}.${pad(date.getSeconds())}`;
}
