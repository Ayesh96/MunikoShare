import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import { SETTINGS_COMMANDS } from "../constants/settings";
import { SETTINGS_PICK_FOLDER_TITLE } from "../messages/settings";
import type { Settings } from "../types/settings";

export const getSettings = () => invoke<Settings>(SETTINGS_COMMANDS.get);
export const setDeviceName = (name: string) => invoke<Settings>(SETTINGS_COMMANDS.setDeviceName, { name });
export const setDownloadDir = (path: string | null) => invoke<Settings>(SETTINGS_COMMANDS.setDownloadDir, { path });
export const openDownloadDir = () => invoke<void>(SETTINGS_COMMANDS.openDownloadDir);
export const takeLaunchPaths = () => invoke<string[]>(SETTINGS_COMMANDS.takeLaunchPaths);

/** Resolves to the chosen folder, or null if the dialog was cancelled. */
export async function chooseDownloadFolder(): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false, title: SETTINGS_PICK_FOLDER_TITLE });
  return typeof selected === "string" ? selected : null;
}

export { getVersion as getAppVersion } from "@tauri-apps/api/app";
