import { useCallback, useState } from "react";

import type { Notice, Tone } from "../types/ui";

export interface NoticeState {
  notice: Notice | null;
  show: (message: string, tone: Tone) => void;
  clear: () => void;
}

/** One dismissible banner message for action results. */
export function useNotice(): NoticeState {
  const [notice, setNotice] = useState<Notice | null>(null);
  const show = useCallback((message: string, tone: Tone) => setNotice({ message, tone }), []);
  const clear = useCallback(() => setNotice(null), []);
  return { notice, show, clear };
}
