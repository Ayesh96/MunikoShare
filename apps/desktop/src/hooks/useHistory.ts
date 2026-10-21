import { useCallback } from "react";

import { HISTORY_REFRESH_EVENTS } from "../constants/app";
import { clearHistory, getHistory } from "../lib/tauriApi";
import type { HistoryEntry } from "../types/history";
import { useLoaded } from "./useLoaded";

export interface History {
  entries: HistoryEntry[];
  clear: () => Promise<void>;
}

/** Persistent history from the core, reloaded when something worth recording happens. */
export function useHistory(): History {
  const { value, reload } = useLoaded(getHistory, HISTORY_REFRESH_EVENTS);
  const clear = useCallback(async () => {
    await clearHistory();
    reload();
  }, [reload]);
  return { entries: value ?? [], clear };
}
