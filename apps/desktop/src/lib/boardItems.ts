import { IMAGE_EXTENSION, IMAGE_PREVIEW_MAX_BYTES } from "../constants/board";
import type { BoardItem } from "../types/board";

export function isImageItem(item: BoardItem): boolean {
  return item.kind.type === "file" && item.kind.name.toLowerCase().endsWith(IMAGE_EXTENSION);
}

/** Small enough to preview, and some online device (or this one) has it. */
export function canPreviewImage(item: BoardItem): boolean {
  return isImageItem(item) && item.kind.type === "file" && item.kind.size <= IMAGE_PREVIEW_MAX_BYTES && (item.available || item.heldLocally);
}
