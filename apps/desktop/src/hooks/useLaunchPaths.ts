import { useCallback, useEffect, useState } from "react";

import { onLaunchPaths } from "../lib/tauriApi";
import { takeLaunchPaths } from "../lib/settingsApi";

export interface LaunchPaths {
  paths: string[];
  add: (paths: string[]) => void;
  dismiss: () => void;
}

/** Paths waiting for a send target: from the command line, a forwarded launch, or a window drop. */
export function useLaunchPaths(): LaunchPaths {
  const [paths, setPaths] = useState<string[]>([]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    const drain = () =>
      takeLaunchPaths()
        .then((taken) => !cancelled && taken.length > 0 && setPaths((current) => [...current, ...taken]))
        .catch(() => undefined);
    onLaunchPaths(drain).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    // Drain after subscribing, so a path queued before mount is still picked up.
    drain();
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return {
    paths,
    add: useCallback((added: string[]) => setPaths((current) => [...current, ...added]), []),
    dismiss: useCallback(() => setPaths([]), []),
  };
}
