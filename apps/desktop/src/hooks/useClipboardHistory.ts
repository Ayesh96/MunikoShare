import { useCallback, useEffect } from "react";

import { CLIPBOARD_POLL_INTERVAL_MS, CLIPBOARD_REFRESH_EVENTS } from "../constants/events";
import { errorMessage } from "../lib/format";
import { clearClipboardHistory, clipboardHistory, copyClipboardItem, pinClipboardItem } from "../lib/tauriApi";
import type { ClipboardItem } from "../types/clipboard";
import type { Tone } from "../types/ui";
import { useLoaded } from "./useLoaded";

export interface ClipboardHistoryState {
  items: ClipboardItem[];
  copy: (item: ClipboardItem) => Promise<void>;
  togglePin: (item: ClipboardItem) => Promise<void>;
  clear: () => Promise<void>;
}

/** Clipboard history, reloaded on clipboard events and on a timer while the hook is mounted. */
export function useClipboardHistory(show: (message: string, tone: Tone) => void): ClipboardHistoryState {
  const { value, reload } = useLoaded(clipboardHistory, CLIPBOARD_REFRESH_EVENTS);

  useEffect(() => {
    const timer = setInterval(reload, CLIPBOARD_POLL_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [reload]);

  const copy = useCallback(
    async (item: ClipboardItem) => {
      try {
        await copyClipboardItem(item.id);
      } catch (error) {
        show(errorMessage(error), "danger");
      }
    },
    [show],
  );

  const togglePin = useCallback(
    async (item: ClipboardItem) => {
      try {
        await pinClipboardItem(item.id, !item.pinned);
      } catch (error) {
        show(errorMessage(error), "danger");
      }
      reload();
    },
    [reload, show],
  );

  const clear = useCallback(async () => {
    try {
      await clearClipboardHistory();
    } catch (error) {
      show(errorMessage(error), "danger");
    }
    reload();
  }, [reload, show]);

  return { items: value ?? [], copy, togglePin, clear };
}
