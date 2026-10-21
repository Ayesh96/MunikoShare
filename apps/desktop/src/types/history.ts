// Mirrors muniko_core::history::HistoryEntry (serde camelCase, timestamp in unix seconds).
export type HistoryKind =
  | "fileReceived"
  | "fileSent"
  | "transferFailed"
  | "clipboardReceived"
  | "clipboardSent"
  | "paired"
  | "unpaired";

export interface HistoryEntry {
  id: number;
  timestamp: number;
  kind: HistoryKind;
  deviceId: string;
  detail: string;
}
