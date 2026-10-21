import { useEffect, useState } from "react";

import { COUNTDOWN_TICK_MS } from "../constants/events";

/** Seconds left until a unix-second deadline, ticking once a second while mounted. */
export function useCountdown(expiresAt: number | null): number {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), COUNTDOWN_TICK_MS);
    return () => clearInterval(timer);
  }, []);

  return expiresAt === null ? 0 : Math.max(0, Math.ceil(expiresAt - now / 1000));
}
