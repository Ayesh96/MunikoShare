import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

/** Resolves to the pending update, or null when already up to date; rejects when the endpoint is unreachable. */
export const checkForUpdate = (): Promise<Update | null> => check();

export async function installUpdate(update: Update): Promise<void> {
  await update.downloadAndInstall();
  await relaunch();
}
