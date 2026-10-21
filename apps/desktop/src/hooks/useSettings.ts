import { useCallback } from "react";

import { chooseDownloadFolder, getSettings, openDownloadDir, setDeviceName, setDownloadDir } from "../lib/settingsApi";
import { SETTINGS_FOLDER_SAVED, SETTINGS_NAME_SAVED } from "../messages/settings";
import type { Settings } from "../types/settings";
import type { Tone } from "../types/ui";
import { useLoaded } from "./useLoaded";
import { useRun } from "./useRun";

export interface SettingsActions {
  settings: Settings | null;
  saveName: (name: string) => Promise<void>;
  chooseFolder: () => Promise<void>;
  resetFolder: () => Promise<void>;
  openFolder: () => Promise<void>;
}

/** Saved preferences; they apply on the next start. */
export function useSettings(show: (message: string, tone: Tone) => void): SettingsActions {
  const { value, reload } = useLoaded(getSettings);
  const run = useRun(show);

  const saveAndReload = useCallback(
    async (save: () => Promise<unknown>, success: string) => {
      if (await run(save, success)) reload();
    },
    [run, reload],
  );

  return {
    settings: value,
    saveName: (name) => saveAndReload(() => setDeviceName(name), SETTINGS_NAME_SAVED),
    chooseFolder: async () => {
      const folder = await chooseDownloadFolder();
      if (folder) await saveAndReload(() => setDownloadDir(folder), SETTINGS_FOLDER_SAVED);
    },
    resetFolder: () => saveAndReload(() => setDownloadDir(null), SETTINGS_FOLDER_SAVED),
    openFolder: async () => void (await run(openDownloadDir)),
  };
}
