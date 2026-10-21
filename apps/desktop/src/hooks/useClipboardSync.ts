import { useCallback, useMemo } from "react";

import { PAIRING_CHANGE_EVENTS } from "../constants/events";
import { errorMessage } from "../lib/format";
import { listClipboardSync, setClipboardSync } from "../lib/tauriApi";
import type { Tone } from "../types/ui";
import { useLoaded } from "./useLoaded";

export interface ClipboardSync {
  enabled: Set<string>;
  toggle: (deviceId: string, enabled: boolean) => Promise<void>;
}

/** Devices whose clipboard is synced automatically. */
export function useClipboardSync(show: (message: string, tone: Tone) => void): ClipboardSync {
  const { value, reload } = useLoaded(listClipboardSync, PAIRING_CHANGE_EVENTS);
  const enabled = useMemo(() => new Set(value ?? []), [value]);

  const toggle = useCallback(
    async (deviceId: string, next: boolean) => {
      try {
        await setClipboardSync(deviceId, next);
      } catch (error) {
        show(errorMessage(error), "danger");
      }
      reload();
    },
    [reload, show],
  );

  return { enabled, toggle };
}
