// Mirrors muniko_core::board (serde camelCase; timestamps in unix seconds).
export type BoardMode = "open" | "protected";

export interface Board {
  id: string;
  name: string;
  mode: BoardMode;
  createdAt: number;
}

export interface ObservedBoard {
  peerId: string;
  peerName: string;
  info: Board & { memberCount: number };
}

export type BoardItemKind =
  | { type: "text"; text: string }
  | { type: "file"; name: string; size: number; sha256: string };

export interface BoardItem {
  id: string;
  authorId: string;
  authorName: string;
  createdAt: number;
  mine: boolean;
  kind: BoardItemKind;
  heldLocally: boolean;
  available: boolean;
}

export interface BoardState {
  board: Board | null;
  items: BoardItem[];
  conflicts: ObservedBoard[];
  discoveredBoards: ObservedBoard[];
  keeper: boolean;
}
