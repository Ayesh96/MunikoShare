import { useCallback, useState } from "react";

import { BOARD_REFRESH_DEBOUNCE_MS, BOARD_REFRESH_EVENTS } from "../constants/events";
import { copyText } from "../lib/clipboard";
import {
  boardCreate,
  boardDelete,
  boardDownload,
  boardJoin,
  boardLeave,
  boardPostFiles,
  boardPostText,
  boardSetKeeper,
  boardState,
} from "../lib/boardApi";
import { pickFiles } from "../lib/tauriApi";
import { BOARD_COPIED, BOARD_PICK_FILES_TITLE, boardDownloaded } from "../messages/board";
import type { BoardItem, BoardMode, BoardState, ObservedBoard } from "../types/board";
import type { Tone } from "../types/ui";
import { useDebouncedNodeEvents } from "./useDebouncedNodeEvents";
import { useFileDrop } from "./useFileDrop";
import { useLoaded } from "./useLoaded";
import { useRun } from "./useRun";

/** A join that failed after the previous board was already left. */
export interface FailedJoin {
  target: ObservedBoard;
  code?: string;
}

export interface BoardActions {
  state: BoardState | null;
  failedJoin: FailedJoin | null;
  retryJoin: () => Promise<void>;
  dismissFailedJoin: () => void;
  postText: (text: string) => Promise<boolean>;
  addFiles: () => Promise<void>;
  copy: (item: BoardItem) => Promise<void>;
  download: (item: BoardItem) => Promise<void>;
  remove: (item: BoardItem) => Promise<void>;
  create: (name: string, mode: BoardMode, joinCode?: string) => Promise<boolean>;
  join: (target: ObservedBoard, code?: string, replace?: boolean) => Promise<boolean>;
  leave: () => Promise<void>;
  setKeeper: (keeper: boolean) => Promise<void>;
}

/** Board state plus its actions; reloads (debounced) on board events and after every action. */
export function useBoard(show: (message: string, tone: Tone) => void): BoardActions {
  const { value, reload } = useLoaded(boardState);
  const run = useRun(show);
  const [failedJoin, setFailedJoin] = useState<FailedJoin | null>(null);
  useDebouncedNodeEvents(BOARD_REFRESH_EVENTS, BOARD_REFRESH_DEBOUNCE_MS, reload);

  const act = useCallback(
    async (action: () => Promise<unknown>, success?: string) => {
      const ok = await run(action, success);
      reload();
      return ok;
    },
    [run, reload],
  );

  const postFiles = useCallback((paths: string[]) => act(() => boardPostFiles(paths)), [act]);
  useFileDrop((paths) => paths.length > 0 && postFiles(paths));

  // The core cannot join while on a board, so a replacing join leaves first and keeps the target for a retry.
  const join = async (target: ObservedBoard, code?: string, replace = false) => {
    const left = replace && value?.board != null;
    const ok = await act(async () => {
      if (left) await boardLeave();
      await boardJoin(target.peerId, code);
    });
    setFailedJoin(ok || (!left && !failedJoin) ? null : { target, code });
    return ok;
  };

  return {
    state: value,
    failedJoin,
    retryJoin: async () => void (failedJoin && (await join(failedJoin.target, failedJoin.code))),
    dismissFailedJoin: () => setFailedJoin(null),
    postText: (text) => act(() => boardPostText(text)),
    addFiles: async () => {
      const paths = await pickFiles(BOARD_PICK_FILES_TITLE);
      if (paths.length > 0) await postFiles(paths);
    },
    copy: async (item) => {
      const { kind } = item;
      if (kind.type === "text") await run(() => copyText(kind.text), BOARD_COPIED);
    },
    download: async (item) => {
      await run(async () => show(boardDownloaded(await boardDownload(item.id)), "success"));
      reload();
    },
    remove: async (item) => void (await act(() => boardDelete(item.id))),
    create: (name, mode, joinCode) => act(() => boardCreate(name, mode, joinCode)),
    join,
    leave: async () => void (await act(() => boardLeave())),
    setKeeper: async (keeper) => void (await act(() => boardSetKeeper(keeper))),
  };
}
