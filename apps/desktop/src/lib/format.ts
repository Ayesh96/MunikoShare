import { BYTE_UNITS, BYTES_PER_UNIT, CLIPBOARD_PREVIEW_LENGTH, MAX_PORT, MIN_PORT, SHORT_ID_LENGTH } from "../constants/app";

/** Human-readable size such as "1.5 MB". */
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

export function previewText(text: string): string {
  const flat = text.replace(/\s+/g, " ").trim();
  return flat.length > CLIPBOARD_PREVIEW_LENGTH ? `${flat.slice(0, CLIPBOARD_PREVIEW_LENGTH)}…` : flat;
}

/** Tauri rejects with a plain string; fall back for real Error objects. */
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function shortId(deviceId: string): string {
  return deviceId.slice(0, SHORT_ID_LENGTH);
}

/** Parses a TCP port, or returns null when it is not a whole number in range. */
export function parsePort(text: string): number | null {
  const port = Number(text.trim());
  return Number.isInteger(port) && port >= MIN_PORT && port <= MAX_PORT ? port : null;
}
