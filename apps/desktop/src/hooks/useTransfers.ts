import { useCallback, useState } from "react";

import type { NodeEvent } from "../types/events";
import type { ActiveTransfer } from "../types/transfer";
import { useNodeEvents } from "./useNodeEvents";

/** Transfers currently in flight; finished or failed ones are dropped. */
export function useTransfers(): ActiveTransfer[] {
  const [transfers, setTransfers] = useState<ActiveTransfer[]>([]);

  const onEvent = useCallback((event: NodeEvent) => {
    if (event.type === "transferProgress") {
      const { type: _type, ...transfer } = event;
      const finished = transfer.bytes_total > 0 && transfer.bytes_done >= transfer.bytes_total;
      setTransfers((current) => {
        const others = current.filter((item) => item.transfer_id !== transfer.transfer_id);
        return finished ? others : [...others, transfer];
      });
    }
    if (event.type === "transferFailed") {
      setTransfers((current) => current.filter((item) => item.device_id !== event.device_id));
    }
  }, []);
  useNodeEvents(onEvent);

  return transfers;
}
