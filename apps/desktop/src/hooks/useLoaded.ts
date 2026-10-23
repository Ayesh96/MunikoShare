import { useCallback, useEffect, useRef, useState } from "react";

import { useNodeEvents } from "./useNodeEvents";

export interface Loaded<T> {
  value: T | null;
  failed: boolean;
  reload: () => void;
}

/** Loads a value on mount and again whenever one of the listed node events arrives; `loader` must be stable. */
export function useLoaded<T>(loader: () => Promise<T>, reloadOn: readonly string[] = []): Loaded<T> {
  const [value, setValue] = useState<T | null>(null);
  const [failed, setFailed] = useState(false);
  const mounted = useRef(true);
  const latest = useRef(0);

  const reload = useCallback(() => {
    const request = ++latest.current;
    // Only the newest request may set state, so slow older replies cannot overwrite it.
    loader()
      .then((result) => {
        if (!mounted.current || request !== latest.current) return;
        setValue(result);
        setFailed(false);
      })
      .catch(() => mounted.current && request === latest.current && setFailed(true));
  }, [loader]);

  useEffect(() => {
    mounted.current = true;
    reload();
    return () => {
      mounted.current = false;
    };
  }, [reload]);

  useNodeEvents((event) => {
    if (reloadOn.includes(event.type)) reload();
  });

  return { value, failed, reload };
}
