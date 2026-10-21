import { useCallback, useEffect, useState } from "react";

import { api } from "../lib/api";
import { bootstrapSession, resetBootstrap } from "../lib/sessionBootstrap";

export type SessionState = { status: "loading" } | { status: "scan" } | { status: "connected"; deviceName: string };

export function useSession() {
  const [state, setState] = useState<SessionState>({ status: "loading" });
  const lose = useCallback(() => {
    resetBootstrap();
    setState({ status: "scan" });
  }, []);

  useEffect(() => {
    let active = true;
    api.setUnauthorizedHandler(lose);
    bootstrapSession().then((info) => {
      if (active) setState(info ? { status: "connected", deviceName: info.deviceName } : { status: "scan" });
    });
    return () => {
      active = false;
      api.setUnauthorizedHandler(() => {});
    };
  }, [lose]);

  const disconnect = useCallback(async () => {
    await api.deleteSession().catch(() => {});
    lose();
  }, [lose]);

  return { state, disconnect, lose };
}
