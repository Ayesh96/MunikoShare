import { useEffect, useRef } from "react";

/** Runs `tick` every `delayMs` while mounted; the timer is cleared on unmount. */
export function useInterval(tick: () => void, delayMs: number): void {
  const tickRef = useRef(tick);
  tickRef.current = tick;

  useEffect(() => {
    const timer = setInterval(() => tickRef.current(), delayMs);
    return () => clearInterval(timer);
  }, [delayMs]);
}
