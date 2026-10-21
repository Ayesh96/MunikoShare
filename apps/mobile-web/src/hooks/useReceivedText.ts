import { useCallback, useEffect, useState } from "react";

import { api } from "../lib/api";
import type { SharedText } from "../types/api";

export function useReceivedText() {
  const [shared, setShared] = useState<SharedText | null>(null);
  const refresh = useCallback(() => {
    api.getText().then(setShared).catch(() => {});
  }, []);
  useEffect(refresh, [refresh]);
  return { shared, refresh };
}
