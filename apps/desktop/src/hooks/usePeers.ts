import { useCallback, useEffect, useState } from "react";

import { listPeers } from "../lib/tauriApi";
import type { DeviceInfo } from "../types/device";
import type { NodeEvent } from "../types/events";
import { useNodeEvents } from "./useNodeEvents";

function upsert(peers: DeviceInfo[], device: DeviceInfo): DeviceInfo[] {
  return [...peers.filter((peer) => peer.id !== device.id), device];
}

export interface Peers {
  peers: DeviceInfo[];
  loading: boolean;
}

export function usePeers(): Peers {
  const [peers, setPeers] = useState<DeviceInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    // Merge so a PeerFound that arrived before the initial load is kept.
    listPeers()
      .then((initial) => !cancelled && setPeers((current) => initial.reduce(upsert, current)))
      .catch(() => undefined)
      .finally(() => !cancelled && setLoading(false));
    return () => {
      cancelled = true;
    };
  }, []);

  const onEvent = useCallback((event: NodeEvent) => {
    if (event.type === "peerFound" || event.type === "peerAdded") setPeers((current) => upsert(current, event.device));
    if (event.type === "peerLost") setPeers((current) => current.filter((peer) => peer.id !== event.device_id));
  }, []);
  useNodeEvents(onEvent);

  return { peers, loading };
}
