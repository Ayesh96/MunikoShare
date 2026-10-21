import { useCallback } from "react";

import { errorMessage } from "../lib/format";
import type { Tone } from "../types/ui";

/** Runs an action and reports failures through `show`; resolves true on success. */
export function useRun(show: (message: string, tone: Tone) => void): (action: () => Promise<unknown>, success?: string) => Promise<boolean> {
  return useCallback(
    async (action, success) => {
      try {
        await action();
        if (success) show(success, "success");
        return true;
      } catch (error) {
        show(errorMessage(error), "danger");
        return false;
      }
    },
    [show],
  );
}
