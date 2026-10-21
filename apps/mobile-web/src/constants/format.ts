export const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB"] as const;
export const BYTES_PER_UNIT = 1024;
export const SECONDS_PER_MINUTE = 60;
export const SECONDS_PER_HOUR = 3600;
export const SECONDS_PER_DAY = 86400;
export const MS_PER_SECOND = 1000;
export const COPIED_RESET_MS = 2000;
// Values above this are milliseconds, not unix seconds.
export const UNIX_MS_THRESHOLD = 1e11;
