import { useCallback, useEffect, useState } from "react";

import { api } from "../lib/api";
import type { BoardResponse } from "../types/api";

export function useBoard() {
  const [data, setData] = useState<BoardResponse | null>(null);
  const refresh = useCallback(() => {
    api.getBoard().then(setData).catch(() => {});
  }, []);
  useEffect(refresh, [refresh]);
  return { data, refresh };
}
