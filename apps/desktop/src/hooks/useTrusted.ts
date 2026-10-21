import { useMemo } from "react";

import { PAIRING_CHANGE_EVENTS } from "../constants/events";
import { listTrusted } from "../lib/tauriApi";
import type { TrustedDevice } from "../types/trust";
import { useLoaded } from "./useLoaded";

export interface Trusted {
  ids: Set<string>;
  devices: TrustedDevice[];
}

/** Paired devices, reloaded whenever pairing changes. */
export function useTrusted(): Trusted {
  const devices = useLoaded(listTrusted, PAIRING_CHANGE_EVENTS).value;
  return useMemo(() => {
    const list = devices ?? [];
    return { ids: new Set(list.map((device) => device.id)), devices: list };
  }, [devices]);
}
