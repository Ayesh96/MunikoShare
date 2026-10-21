import { invoke } from "@tauri-apps/api/core";

import { BOARD_COMMANDS, PHONE_COMMANDS } from "../constants/board";
import type { BoardMode, BoardState } from "../types/board";
import type { PhoneInvite, PhoneSession } from "../types/phone";

export const boardState = () => invoke<BoardState>(BOARD_COMMANDS.state);
export const boardPostText = (text: string) => invoke<void>(BOARD_COMMANDS.postText, { text });
export const boardPostFiles = (paths: string[]) => invoke<void>(BOARD_COMMANDS.postFiles, { paths });
export const boardDownload = (itemId: string) => invoke<string>(BOARD_COMMANDS.download, { itemId });
export const boardDelete = (itemId: string) => invoke<void>(BOARD_COMMANDS.delete, { itemId });
export const boardCreate = (name: string, mode: BoardMode, joinCode?: string) =>
  invoke<void>(BOARD_COMMANDS.create, { name, mode, joinCode });
export const boardJoin = (peerId: string, code?: string) => invoke<void>(BOARD_COMMANDS.join, { peerId, code });
export const boardLeave = () => invoke<void>(BOARD_COMMANDS.leave);
export const boardSetKeeper = (keeper: boolean) => invoke<void>(BOARD_COMMANDS.setKeeper, { keeper });

export const phoneInvite = () => invoke<PhoneInvite>(PHONE_COMMANDS.invite);
export const phoneSessions = () => invoke<PhoneSession[]>(PHONE_COMMANDS.sessions);
export const phoneRevoke = (sessionId: string) => invoke<boolean>(PHONE_COMMANDS.revoke, { sessionId });
export const phoneSendFiles = (sessionId: string, paths: string[]) =>
  invoke<number>(PHONE_COMMANDS.sendFiles, { sessionId, paths });
export const phoneShareText = (sessionId: string, text: string) =>
  invoke<void>(PHONE_COMMANDS.shareText, { sessionId, text });
export const phoneSetAccess = (enabled: boolean) => invoke<void>(PHONE_COMMANDS.setAccess, { enabled });
export const phoneAccess = () => invoke<boolean>(PHONE_COMMANDS.access);
