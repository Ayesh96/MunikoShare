import { useEffect, useRef } from "react";

import { onNodeEvent } from "../lib/tauriApi";
import type { NodeEvent } from "../types/events";

/** Subscribes to node events and unlistens on unmount, even if unmounted before listen resolves. */
export function useNodeEvents(handler: (event: NodeEvent) => void): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    onNodeEvent((event) => handlerRef.current(event)).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);
}
