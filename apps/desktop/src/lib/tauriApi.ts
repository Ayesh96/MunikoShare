import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";

import { COMMANDS, HISTORY_LIMIT, HOTKEY_EVENT_NAME, LAUNCH_EVENT_NAME, NODE_EVENT_NAME } from "../constants/app";
import { PICK_FILES_TITLE, PICK_FOLDER_TITLE } from "../messages/ui";
import type { ClipboardItem, HotkeyResult } from "../types/clipboard";
import type { DeviceInfo } from "../types/device";
import type { NodeEvent } from "../types/events";
import type { HistoryEntry } from "../types/history";
import type { TrustedDevice } from "../types/trust";

export const getLocalDevice = () => invoke<DeviceInfo>(COMMANDS.getLocalDevice);
export const listPeers = () => invoke<DeviceInfo[]>(COMMANDS.listPeers);
export const listTrusted = () => invoke<TrustedDevice[]>(COMMANDS.listTrusted);
export const localAddresses = () => invoke<string[]>(COMMANDS.localAddresses);
export const addPeerByAddress = (ip: string, port: number) =>
  invoke<DeviceInfo>(COMMANDS.addPeerByAddress, { ip, port });

export const sendFiles = (peerId: string, paths: string[]) =>
  invoke<void>(COMMANDS.sendFiles, { peerId, paths });
export const respondToOffer = (offerId: string, accept: boolean) =>
  invoke<void>(COMMANDS.respondToOffer, { offerId, accept });

export const requestPairing = (peerId: string) => invoke<void>(COMMANDS.requestPairing, { peerId });
export const confirmPairing = (peerId: string, code: string) =>
  invoke<void>(COMMANDS.confirmPairing, { peerId, code });
export const unpair = (deviceId: string) => invoke<void>(COMMANDS.unpair, { deviceId });
export const setAutoAccept = (deviceId: string, enabled: boolean) =>
  invoke<void>(COMMANDS.setAutoAccept, { deviceId, enabled });
export const listAutoAccept = () => invoke<string[]>(COMMANDS.listAutoAccept);

export const getHistory = (limit = HISTORY_LIMIT) => invoke<HistoryEntry[]>(COMMANDS.getHistory, { limit });
export const clearHistory = () => invoke<void>(COMMANDS.clearHistory);

export const pushClipboard = (peerId: string) => invoke<void>(COMMANDS.pushClipboard, { peerId });
export const pushText = (peerId: string, text: string) => invoke<void>(COMMANDS.pushText, { peerId, text });

export const setClipboardSync = (deviceId: string, enabled: boolean) =>
  invoke<void>(COMMANDS.setClipboardSync, { deviceId, enabled });
export const listClipboardSync = () => invoke<string[]>(COMMANDS.listClipboardSync);
export const clipboardHistory = () => invoke<ClipboardItem[]>(COMMANDS.clipboardHistory);
export const copyClipboardItem = (id: string) => invoke<void>(COMMANDS.copyClipboardItem, { id });
export const clipboardImage = (id: string) => invoke<string | null>(COMMANDS.clipboardImage, { id });
export const pinClipboardItem = (id: string, pinned: boolean) =>
  invoke<boolean>(COMMANDS.pinClipboardItem, { id, pinned });
export const clearClipboardHistory = () => invoke<void>(COMMANDS.clearClipboardHistory);

export const onHotkeyResult = (handler: (result: HotkeyResult) => void): Promise<UnlistenFn> =>
  listen<HotkeyResult>(HOTKEY_EVENT_NAME, (message) => handler(message.payload));

export const onNodeEvent = (handler: (event: NodeEvent) => void): Promise<UnlistenFn> =>
  listen<NodeEvent>(NODE_EVENT_NAME, (message) => handler(message.payload));

/** Fires with file paths dropped onto the window. */
export const onFilesDropped = (handler: (paths: string[]) => void): Promise<UnlistenFn> =>
  getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === "drop") handler(event.payload.paths);
  });

function asPaths(selected: string | string[] | null): string[] {
  if (Array.isArray(selected)) return selected;
  return typeof selected === "string" ? [selected] : [];
}

/** Resolves to the chosen paths, or an empty list if the dialog was cancelled. */
export async function pickFiles(title = PICK_FILES_TITLE): Promise<string[]> {
  return asPaths(await open({ multiple: true, directory: false, title }));
}

export async function pickFolder(): Promise<string[]> {
  return asPaths(await open({ multiple: false, directory: true, title: PICK_FOLDER_TITLE }));
}

/** Fires when the shell queued paths from launch arguments; drain them with `takeLaunchPaths`. */
export const onLaunchPaths = (handler: () => void): Promise<UnlistenFn> => listen(LAUNCH_EVENT_NAME, handler);
