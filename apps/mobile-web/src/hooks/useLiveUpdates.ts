import { useEffect, useRef } from "react";

import { EVENT_BOARD, EVENT_INBOX, EVENT_REVOKED, EVENT_TEXT, EVENTS_PATH, POLL_INTERVAL_MS } from "../constants/api";

export interface LiveHandlers {
  onInbox: () => void;
  onText: () => void;
  onBoard: () => void;
  onRevoked: () => void;
}

/** Server-sent events, falling back to polling when the stream errors. */
export function useLiveUpdates(handlers: LiveHandlers): void {
  const ref = useRef(handlers);
  ref.current = handlers;

  useEffect(() => {
    let poll: number | undefined;
    const source = new EventSource(EVENTS_PATH);
    const startPolling = () => {
      source.close();
      if (poll !== undefined) return;
      poll = window.setInterval(() => {
        ref.current.onInbox();
        ref.current.onText();
        ref.current.onBoard();
      }, POLL_INTERVAL_MS);
    };
    const onInbox = () => ref.current.onInbox();
    const onText = () => ref.current.onText();
    const onBoard = () => ref.current.onBoard();
    const onRevoked = () => ref.current.onRevoked();
    source.addEventListener(EVENT_INBOX, onInbox);
    source.addEventListener(EVENT_TEXT, onText);
    source.addEventListener(EVENT_BOARD, onBoard);
    source.addEventListener(EVENT_REVOKED, onRevoked);
    source.onerror = startPolling;
    return () => {
      source.removeEventListener(EVENT_INBOX, onInbox);
      source.removeEventListener(EVENT_TEXT, onText);
      source.removeEventListener(EVENT_BOARD, onBoard);
      source.removeEventListener(EVENT_REVOKED, onRevoked);
      source.close();
      if (poll !== undefined) window.clearInterval(poll);
    };
  }, []);
}
