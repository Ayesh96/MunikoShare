import { BOARD_FILES_EMPTY_ICON } from "../constants/icons";
import { useBoard } from "../hooks/useBoard";
import { useFileDrop } from "../hooks/useFileDrop";
import { BOARD_FILES_EMPTY_HINT, BOARD_FILES_EMPTY_TITLE, BOARD_FILES_TITLE } from "../messages/board";
import type { Tone } from "../types/ui";
import { BoardFeed } from "./BoardFeed";
import { BoardGate } from "./BoardGate";
import { ShareFilesCard } from "./ShareFilesCard";

export interface BoardTabProps {
  show: (message: string, tone: Tone) => void;
  onOpenSettings: () => void;
}

/** Open file sharing: every device on the board sees the files. Mounted only while visible, so drops post only from here. */
export function BoardTab({ show, onOpenSettings }: BoardTabProps) {
  const board = useBoard(show);
  useFileDrop((paths) => paths.length > 0 && board.postFiles(paths));
  const files = board.state?.items.filter((item) => item.kind.type === "file") ?? [];

  return (
    <BoardGate state={board.state} onOpenSettings={onOpenSettings}>
      <ShareFilesCard onAddFiles={board.addFiles} />
      <BoardFeed
        title={BOARD_FILES_TITLE}
        emptyTitle={BOARD_FILES_EMPTY_TITLE}
        emptyHint={BOARD_FILES_EMPTY_HINT}
        emptyIcon={BOARD_FILES_EMPTY_ICON}
        items={files}
        onCopy={board.copy}
        onDownload={board.download}
        onDelete={board.remove}
      />
    </BoardGate>
  );
}
