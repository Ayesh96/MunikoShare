// Mirrors muniko_core::clipboard::HistorySummary (serde camelCase, timestamp in unix seconds).
export type ClipboardSource = { type: "local" } | { type: "device"; deviceId: string };

export type ClipboardPreview =
  | { type: "text"; preview: string; length: number }
  | { type: "image"; width: number; height: number; bytes: number };

export interface ClipboardItem {
  id: string;
  source: ClipboardSource;
  timestamp: number;
  pinned: boolean;
  preview: ClipboardPreview;
}

// Payload of the global hotkey result event.
export interface HotkeyResult {
  sent: number;
  errors: string[];
}
