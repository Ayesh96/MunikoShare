import { useCallback, useState } from "react";

import { addPeerByAddress } from "../lib/tauriApi";
import { deviceAdded } from "../messages/ui";
import type { Tone } from "../types/ui";

export interface AddDevice {
  isOpen: boolean;
  open: () => void;
  close: () => void;
  /** Rejects with the error text so the form can show it. */
  add: (ip: string, port: number) => Promise<void>;
}

export function useAddDevice(show: (message: string, tone: Tone) => void): AddDevice {
  const [isOpen, setIsOpen] = useState(false);
  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);

  const add = useCallback(
    async (ip: string, port: number) => {
      const device = await addPeerByAddress(ip, port);
      show(deviceAdded(device.name), "success");
      setIsOpen(false);
    },
    [show],
  );

  return { isOpen, open, close, add };
}
