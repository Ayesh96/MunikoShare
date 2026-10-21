import { useCallback, useState } from "react";

import type { NodeEvent } from "../types/events";
import { useNodeEvents } from "./useNodeEvents";

export interface PendingSends {
  /** Device ids we offered files to that have not started receiving yet. */
  waiting: string[];
  track: (deviceId: string, task: () => Promise<void>) => Promise<void>;
}

function endedDevice(event: NodeEvent): string | null {
  if (event.type === "transferProgress" && event.direction === "outgoing") return event.device_id;
  if (event.type === "transferFailed") return event.device_id;
  if (event.type === "fileSent") return event.to_device_id;
  return null;
}

/** Tracks the wait between offering files and the first progress, failure or completion. */
export function usePendingSends(): PendingSends {
  const [waiting, setWaiting] = useState<string[]>([]);

  const end = useCallback((deviceId: string) => setWaiting((current) => current.filter((id) => id !== deviceId)), []);

  const track = useCallback(
    async (deviceId: string, task: () => Promise<void>) => {
      setWaiting((current) => [...current.filter((id) => id !== deviceId), deviceId]);
      try {
        await task();
      } finally {
        end(deviceId);
      }
    },
    [end],
  );

  useNodeEvents((event) => {
    const deviceId = endedDevice(event);
    if (deviceId) end(deviceId);
  });

  return { waiting, track };
}
