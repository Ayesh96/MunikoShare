import { useCallback, useEffect, useState } from "react";

import { phoneInvite } from "../lib/boardApi";
import { errorMessage } from "../lib/format";
import type { PhoneInvite } from "../types/phone";

export interface PhoneInviteState {
  invite: PhoneInvite | null;
  error: string | null;
  renew: () => void;
}

/** Issues a one-time invite when mounted, and a fresh one on `renew`. */
export function usePhoneInvite(): PhoneInviteState {
  const [invite, setInvite] = useState<PhoneInvite | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [round, setRound] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    phoneInvite()
      .then((result) => !cancelled && setInvite(result))
      .catch((failure) => !cancelled && setError(errorMessage(failure)));
    return () => {
      cancelled = true;
    };
  }, [round]);

  const renew = useCallback(() => setRound((value) => value + 1), []);
  return { invite, error, renew };
}
