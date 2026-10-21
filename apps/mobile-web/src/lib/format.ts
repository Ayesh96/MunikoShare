import { BYTE_UNITS, BYTES_PER_UNIT, MS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, UNIX_MS_THRESHOLD } from "../constants/format";
import { messages } from "../messages/app";

export function formatBytes(bytes: number): string {
  let value = bytes;
  let unit = 0;
  while (value >= BYTES_PER_UNIT && unit < BYTE_UNITS.length - 1) {
    value /= BYTES_PER_UNIT;
    unit += 1;
  }
  const digits = unit === 0 || value >= 10 ? 0 : 1;
  return `${value.toFixed(digits)} ${BYTE_UNITS[unit]}`;
}

/** Accepts unix seconds or milliseconds. */
export function formatRelativeTime(timestamp: number, nowMs: number = Date.now()): string {
  const thenMs = timestamp > UNIX_MS_THRESHOLD ? timestamp : timestamp * MS_PER_SECOND;
  const seconds = Math.max(0, Math.floor((nowMs - thenMs) / MS_PER_SECOND));
  if (seconds < SECONDS_PER_MINUTE) return messages.time.now;
  if (seconds < SECONDS_PER_HOUR) return messages.time.minutes(Math.floor(seconds / SECONDS_PER_MINUTE));
  if (seconds < SECONDS_PER_DAY) return messages.time.hours(Math.floor(seconds / SECONDS_PER_HOUR));
  return messages.time.days(Math.floor(seconds / SECONDS_PER_DAY));
}
