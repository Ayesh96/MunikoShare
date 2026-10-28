import { useCallback } from "react";

import { errorMessage } from "../lib/format";
import { pickFiles, pickFolder, pushClipboard, sendFiles, unpair } from "../lib/tauriApi";
import { clipboardSentText } from "../messages/ui";
import type { DeviceInfo } from "../types/device";
import type { Tone } from "../types/ui";
import type { PendingSends } from "./usePendingSends";

export interface DeviceActions {
  sendFilesTo: (peer: DeviceInfo) => Promise<void>;
  sendPathsTo: (peer: DeviceInfo, paths: string[]) => Promise<void>;
  sendFolderTo: (peer: DeviceInfo) => Promise<void>;
  sendClipboardTo: (peer: DeviceInfo) => Promise<void>;
  unpairDevice: (peer: DeviceInfo) => Promise<void>;
}

/** Per-device commands; failures are reported through `show`. */
export function useDeviceActions(show: (message: string, tone: Tone) => void, pending: PendingSends): DeviceActions {
  const run = useCallback(
    async (action: () => Promise<void>, successMessage?: string) => {
      try {
        await action();
        if (successMessage) show(successMessage, "success");
      } catch (error) {
        show(errorMessage(error), "danger");
      }
    },
    [show],
  );

  const sendPicked = useCallback(
    async (peer: DeviceInfo, pick: () => Promise<string[]>) => {
      const paths = await pick();
      if (paths.length > 0) await run(() => pending.track(peer.id, () => sendFiles(peer.id, paths)));
    },
    [run, pending],
  );

  const sendPathsTo = useCallback(
    (peer: DeviceInfo, paths: string[]) => run(() => pending.track(peer.id, () => sendFiles(peer.id, paths))),
    [run, pending],
  );
  const sendFilesTo = useCallback((peer: DeviceInfo) => sendPicked(peer, pickFiles), [sendPicked]);
  const sendFolderTo = useCallback((peer: DeviceInfo) => sendPicked(peer, pickFolder), [sendPicked]);
  const sendClipboardTo = useCallback(
    (peer: DeviceInfo) => run(() => pushClipboard(peer.id), clipboardSentText(peer.name)),
    [run],
  );
  const unpairDevice = useCallback((peer: DeviceInfo) => run(() => unpair(peer.id)), [run]);

  return { sendPathsTo, sendFilesTo, sendFolderTo, sendClipboardTo, unpairDevice };
}
