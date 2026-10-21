import { useEffect, useState } from "react";

import { COPIED_RESET_MS } from "../constants/format";
import { copyText } from "../lib/clipboard";

/** Copies text and reports `copied` briefly afterwards. */
export function useCopied(text: string) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), COPIED_RESET_MS);
    return () => window.clearTimeout(timer);
  }, [copied]);

  const copy = async () => setCopied(await copyText(text));
  return { copied, copy };
}
