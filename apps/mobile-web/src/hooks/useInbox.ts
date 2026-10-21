import { useCallback, useEffect, useState } from "react";

import { api } from "../lib/api";
import type { InboxItem } from "../types/api";

export function useInbox() {
  const [items, setItems] = useState<InboxItem[]>([]);
  const refresh = useCallback(() => {
    api.listInbox().then(setItems).catch(() => {});
  }, []);
  useEffect(refresh, [refresh]);
  return { items, refresh };
}
