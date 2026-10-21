import { useEffect, useState } from "react";

import { autostartEnabled, setAutostart } from "../lib/autostart";
import type { Tone } from "../types/ui";
import { useRun } from "./useRun";

export interface Autostart {
  /** Null until the first read finishes. */
  enabled: boolean | null;
  set: (enabled: boolean) => Promise<void>;
}

export function useAutostart(show: (message: string, tone: Tone) => void): Autostart {
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const run = useRun(show);

  useEffect(() => {
    let cancelled = false;
    autostartEnabled()
      .then((value) => !cancelled && setEnabled(value))
      .catch(() => !cancelled && setEnabled(false));
    return () => {
      cancelled = true;
    };
  }, []);

  return {
    enabled,
    set: async (next) => {
      if (await run(() => setAutostart(next))) setEnabled(next);
    },
  };
}
