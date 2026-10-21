import { useCallback, useState } from "react";

import { errorMessage } from "../lib/format";
import { respondToOffer } from "../lib/tauriApi";
import type { NodeEvent } from "../types/events";
import type { IncomingOffer } from "../types/transfer";
import { useNodeEvents } from "./useNodeEvents";

export interface IncomingOffers {
  /** The oldest unanswered offer, or null. */
  current: IncomingOffer | null;
  respond: (accept: boolean) => Promise<void>;
}

export function useIncomingOffers(onError: (message: string) => void): IncomingOffers {
  const [offers, setOffers] = useState<IncomingOffer[]>([]);

  const onEvent = useCallback((event: NodeEvent) => {
    if (event.type === "incomingOffer") {
      const { type: _type, ...offer } = event;
      setOffers((current) => [...current, offer]);
    }
    if (event.type === "offerClosed") {
      setOffers((current) => current.filter((offer) => offer.offer_id !== event.offer_id));
    }
  }, []);
  useNodeEvents(onEvent);

  const current = offers[0] ?? null;
  const respond = useCallback(
    async (accept: boolean) => {
      if (!current) return;
      try {
        await respondToOffer(current.offer_id, accept);
      } catch (error) {
        onError(errorMessage(error));
      }
      setOffers((all) => all.filter((offer) => offer.offer_id !== current.offer_id));
    },
    [current, onError],
  );

  return { current, respond };
}
