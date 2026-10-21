import { useCallback, useMemo } from "react";

import { PAIRING_CHANGE_EVENTS } from "../constants/events";
import { errorMessage } from "../lib/format";
import { listAutoAccept, setAutoAccept } from "../lib/tauriApi";
import type { Tone } from "../types/ui";
import { useLoaded } from "./useLoaded";

export interface AutoAccept {
  enabled: Set<string>;
  toggle: (deviceId: string, enabled: boolean) => Promise<void>;
}

/** Devices whose incoming files are accepted without asking. */
export function useAutoAccept(show: (message: string, tone: Tone) => void): AutoAccept {
  const { value, reload } = useLoaded(listAutoAccept, PAIRING_CHANGE_EVENTS);
  const enabled = useMemo(() => new Set(value ?? []), [value]);

  const toggle = useCallback(
    async (deviceId: string, next: boolean) => {
      try {
        await setAutoAccept(deviceId, next);
      } catch (error) {
        show(errorMessage(error), "danger");
      }
      reload();
    },
    [reload, show],
  );

  return { enabled, toggle };
}
