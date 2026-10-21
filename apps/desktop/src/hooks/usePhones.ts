

import { PHONE_REFRESH_EVENTS } from "../constants/events";
import { phoneAccess, phoneRevoke, phoneSendFiles, phoneSessions, phoneSetAccess, phoneShareText } from "../lib/boardApi";
import { pickFiles } from "../lib/tauriApi";
import { PHONE_PICK_FILES_TITLE, PHONE_TEXT_SENT, phoneFilesSent } from "../messages/phone";
import type { PhoneSession } from "../types/phone";
import type { Tone } from "../types/ui";
import { useLoaded } from "./useLoaded";
import { useRun } from "./useRun";

export interface Phones {
  sessions: PhoneSession[];
  /** Null until the first load finishes. */
  access: boolean | null;
  refresh: () => void;
  setAccess: (enabled: boolean) => Promise<void>;
  sendFiles: (session: PhoneSession) => Promise<void>;
  sendPaths: (session: PhoneSession, paths: string[]) => Promise<void>;
  sendText: (session: PhoneSession, text: string) => Promise<boolean>;
  disconnect: (session: PhoneSession) => Promise<void>;
}

/** Connected phones and the phone-access switch; reloads on phone events. */
export function usePhones(show: (message: string, tone: Tone) => void): Phones {
  const sessions = useLoaded(phoneSessions, PHONE_REFRESH_EVENTS);
  const access = useLoaded(phoneAccess);
  const run = useRun(show);
  const { reload } = sessions;

  const sendPaths = async (session: PhoneSession, paths: string[]) => {
    let count = 0;
    const ok = await run(async () => {
      count = await phoneSendFiles(session.id, paths);
    });
    if (ok && count > 0) show(phoneFilesSent(count), "success");
  };

  return {
    sessions: sessions.value ?? [],
    access: access.value,
    refresh: reload,
    setAccess: async (enabled) => {
      await run(() => phoneSetAccess(enabled));
      access.reload();
    },
    sendFiles: async (session) => {
      const paths = await pickFiles(PHONE_PICK_FILES_TITLE);
      if (paths.length > 0) await sendPaths(session, paths);
    },
    sendPaths,
    sendText: (session, text) => run(() => phoneShareText(session.id, text), PHONE_TEXT_SENT),
    disconnect: async (session) => {
      await run(() => phoneRevoke(session.id));
      reload();
    },
  };
}

