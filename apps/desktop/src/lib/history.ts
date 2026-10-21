import {
  BADGE_CLIPBOARD_RECEIVED,
  BADGE_CLIPBOARD_SENT,
  BADGE_FAILED,
  BADGE_PAIRED,
  BADGE_RECEIVED,
  BADGE_SENT,
  BADGE_UNPAIRED,
  HISTORY_CLIPBOARD_SENT,
  HISTORY_PAIRED,
  HISTORY_UNPAIRED,
  historyFileReceived,
  historyFileSent,
} from "../messages/ui";
import type { HistoryEntry, HistoryKind } from "../types/history";
import type { BadgeProps } from "../components/ui";
import { fileName, previewText } from "./format";

export const HISTORY_BADGES: Record<HistoryKind, { label: string; tone: NonNullable<BadgeProps["tone"]> }> = {
  fileReceived: { label: BADGE_RECEIVED, tone: "success" },
  fileSent: { label: BADGE_SENT, tone: "neutral" },
  transferFailed: { label: BADGE_FAILED, tone: "danger" },
  clipboardReceived: { label: BADGE_CLIPBOARD_RECEIVED, tone: "neutral" },
  clipboardSent: { label: BADGE_CLIPBOARD_SENT, tone: "neutral" },
  paired: { label: BADGE_PAIRED, tone: "success" },
  unpaired: { label: BADGE_UNPAIRED, tone: "neutral" },
};

/** Main line of a history row; detail is a path, a reason, a preview or a name depending on the kind. */
export function historyTitle(entry: HistoryEntry): string {
  switch (entry.kind) {
    case "fileReceived":
      return historyFileReceived(fileName(entry.detail));
    case "fileSent":
      return historyFileSent(fileName(entry.detail));
    case "clipboardReceived":
      return previewText(entry.detail);
    case "clipboardSent":
      return HISTORY_CLIPBOARD_SENT;
    case "paired":
      return HISTORY_PAIRED;
    case "unpaired":
      return HISTORY_UNPAIRED;
    default:
      return entry.detail;
  }
}
