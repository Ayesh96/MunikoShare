import { useEffect, useRef } from "react";

import { useNodeEvents } from "./useNodeEvents";

/** Calls `onChange` once, `delayMs` after the last matching node event; pending calls are cancelled on unmount. */
export function useDebouncedNodeEvents(types: readonly string[], delayMs: number, onChange: () => void): void {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const changeRef = useRef(onChange);
  changeRef.current = onChange;

  useNodeEvents((event) => {
    if (!types.includes(event.type)) return;
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => changeRef.current(), delayMs);
  });

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );
}
