import { useCallback, useState } from "react";

import { errorMessage } from "../lib/format";
import { confirmPairing, requestPairing } from "../lib/tauriApi";
import type { DeviceInfo } from "../types/device";
import type { NodeEvent } from "../types/events";
import { useNodeEvents } from "./useNodeEvents";

export interface PairingState {
  /** Device we asked to pair with; the user must type the code it shows. */
  target: DeviceInfo | null;
  /** Peer asking to pair with us, with the code to show. */
  incoming: { device: DeviceInfo; code: string } | null;
  start: (peer: DeviceInfo) => Promise<void>;
  confirm: (code: string) => Promise<void>;
  cancel: () => void;
  dismissIncoming: () => void;
}

export function usePairing(onError: (message: string) => void): PairingState {
  const [target, setTarget] = useState<DeviceInfo | null>(null);
  const [incoming, setIncoming] = useState<PairingState["incoming"]>(null);

  const onEvent = useCallback((event: NodeEvent) => {
    if (event.type === "pairingRequested") setIncoming({ device: event.device, code: event.code });
    if (event.type === "pairingCompleted") {
      setIncoming((current) => (current?.device.id === event.device_id ? null : current));
      setTarget((current) => (current?.id === event.device_id ? null : current));
    }
    if (event.type === "pairingFailed") {
      setIncoming((current) => (current?.device.id === event.device_id ? null : current));
    }
  }, []);
  useNodeEvents(onEvent);

  const start = useCallback(
    async (peer: DeviceInfo) => {
      try {
        await requestPairing(peer.id);
        setTarget(peer);
      } catch (error) {
        onError(errorMessage(error));
      }
    },
    [onError],
  );

  // Rejects on failure so the modal can show the error inline.
  const confirm = useCallback(
    async (code: string) => {
      if (!target) return;
      await confirmPairing(target.id, code);
      setTarget(null);
    },
    [target],
  );

  const cancel = useCallback(() => setTarget(null), []);
  const dismissIncoming = useCallback(() => setIncoming(null), []);

  return { target, incoming, start, confirm, cancel, dismissIncoming };
}
