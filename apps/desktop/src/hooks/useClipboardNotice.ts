import { useCallback } from "react";

import { previewText } from "../lib/format";
import { clipboardImageBanner, clipboardReceivedBanner } from "../messages/ui";
import type { NodeEvent } from "../types/events";
import type { Tone } from "../types/ui";
import { useNodeEvents } from "./useNodeEvents";

/** Announces received clipboard text and images (already placed in the clipboard by the core). */
export function useClipboardNotice(
  nameOf: (deviceId: string) => string,
  show: (message: string, tone: Tone) => void,
): void {
  const onEvent = useCallback(
    (event: NodeEvent) => {
      if (event.type === "clipboardReceived") {
        show(clipboardReceivedBanner(nameOf(event.from_device_id), previewText(event.text)), "info");
      }
      if (event.type === "clipboardImageReceived") {
        show(clipboardImageBanner(nameOf(event.from_device_id), event.width, event.height), "info");
      }
    },
    [nameOf, show],
  );
  useNodeEvents(onEvent);
}
