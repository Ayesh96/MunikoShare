import { useCallback } from "react";

import { HISTORY_REFRESH_EVENTS } from "../constants/app";
import { clearHistory, getHistory } from "../lib/tauriApi";
import type { HistoryEntry } from "../types/history";
import { useLoaded } from "./useLoaded";

export interface History {
  entries: HistoryEntry[];
  loading: boolean;
  failed: boolean;
  reload: () => void;
  clear: () => Promise<void>;
}

/** Persistent history from the core, reloaded when something worth recording happens. */
export function useHistory(): History {
  const { value, failed, reload } = useLoaded(getHistory, HISTORY_REFRESH_EVENTS);
  const clear = useCallback(async () => {
    await clearHistory();
    reload();
  }, [reload]);
  return { entries: value ?? [], loading: value === null && !failed, failed, reload, clear };
}
