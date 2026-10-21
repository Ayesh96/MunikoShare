import { useEffect } from "react";

import { onHotkeyResult } from "../lib/tauriApi";
import { HOTKEY_NO_DEVICES, hotkeySent } from "../messages/ui";
import type { Tone } from "../types/ui";

/** Reports the outcome of the global push-clipboard shortcut. */
export function useHotkeyNotice(show: (message: string, tone: Tone) => void): void {
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    onHotkeyResult((result) => {
      if (result.errors.length > 0) show(result.errors.join("; "), "danger");
      else if (result.sent === 0) show(HOTKEY_NO_DEVICES, "info");
      else show(hotkeySent(result.sent), "success");
    }).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [show]);
}
