export interface SessionInfo {
  deviceName: string;
  sessionId: string;
}

export interface InboxItem {
  id: string;
  name: string;
  size: number;
  addedAt: number;
}

export interface SharedText {
  text: string;
  sentAt: number;
}

export type UploadStatus = "uploading" | "done" | "failed";

export interface UploadEntry {
  id: number;
  file: File;
  status: UploadStatus;
  progress: number;
}

export type Tone = "info" | "success" | "danger";

export type BoardItemKind =
  | { type: "text"; text: string }
  | { type: "file"; name: string; size: number; sha256: string };

export interface BoardItemView {
  id: string;
  authorId: string;
  authorName: string;
  createdAt: number;
  mine: boolean;
  heldLocally: boolean;
  available: boolean;
  kind: BoardItemKind;
}

export interface BoardInfo {
  id: string;
  name: string;
  mode: string;
}

export interface BoardResponse {
  board: BoardInfo | null;
  items: BoardItemView[];
}
