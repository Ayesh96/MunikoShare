import type { Update } from "@tauri-apps/plugin-updater";
import { useRef, useState } from "react";

import { errorMessage } from "../lib/format";
import { checkForUpdate, installUpdate } from "../lib/updater";
import type { UpdateStatus } from "../types/settings";

export interface Updater {
  status: UpdateStatus;
  check: () => Promise<void>;
  install: () => Promise<void>;
}

/** Update check and install; an unreachable endpoint becomes an error status, never a crash. */
export function useUpdater(): Updater {
  const [status, setStatus] = useState<UpdateStatus>({ type: "idle" });
  const pending = useRef<Update | null>(null);

  return {
    status,
    check: async () => {
      setStatus({ type: "checking" });
      try {
        const update = await checkForUpdate();
        pending.current = update;
        setStatus(update ? { type: "available", version: update.version } : { type: "upToDate" });
      } catch (error) {
        setStatus({ type: "error", message: errorMessage(error) });
      }
    },
    install: async () => {
      if (!pending.current) return;
      setStatus({ type: "installing" });
      try {
        await installUpdate(pending.current);
      } catch (error) {
        setStatus({ type: "error", message: errorMessage(error) });
      }
    },
  };
}
