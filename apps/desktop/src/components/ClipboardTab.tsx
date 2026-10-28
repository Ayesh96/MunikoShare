import { useState } from "react";

import { CLIPBOARD_EMPTY_ICON } from "../constants/icons";
import { useBoard } from "../hooks/useBoard";
import { useClipboardHistory } from "../hooks/useClipboardHistory";
import { useImagePaste } from "../hooks/useImagePaste";
import { isImageItem } from "../lib/boardItems";
import { CLIPBOARD_SHARED_EMPTY_HINT, CLIPBOARD_SHARED_EMPTY_TITLE, CLIPBOARD_SHARED_TITLE } from "../messages/ui";
import type { Tone } from "../types/ui";
import { BoardFeed } from "./BoardFeed";
import { BoardGate } from "./BoardGate";
import { ClipboardHistoryCard } from "./ClipboardHistoryCard";
import { ShareCard } from "./ShareCard";

export interface ClipboardTabProps {
  nameOf: (deviceId: string) => string;
  show: (message: string, tone: Tone) => void;
  onOpenSettings: () => void;
}

/** Open shared clipboard: text and pasted images on the board, which everyone on it can copy. Mounted only while visible. */
export function ClipboardTab({ nameOf, show, onOpenSettings }: ClipboardTabProps) {
  const board = useBoard(show);
  const history = useClipboardHistory(show);
  const shared = board.state?.items.filter((item) => item.kind.type === "text" || isImageItem(item)) ?? [];
  const [draftImage, setDraftImage] = useState<Blob | null>(null);
  useImagePaste((image) => board.state?.board && setDraftImage(image));

  return (
    <>
      <BoardGate state={board.state} onOpenSettings={onOpenSettings}>
        <ShareCard
          image={draftImage}
          onRemoveImage={() => setDraftImage(null)}
          onShareImage={board.postImage}
          onShareText={board.postText}
        />
        <BoardFeed
          title={CLIPBOARD_SHARED_TITLE}
          emptyTitle={CLIPBOARD_SHARED_EMPTY_TITLE}
          emptyHint={CLIPBOARD_SHARED_EMPTY_HINT}
          emptyIcon={CLIPBOARD_EMPTY_ICON}
          items={shared}
          onCopy={board.copy}
          onDownload={board.download}
          onDelete={board.remove}
        />
      </BoardGate>
      <ClipboardHistoryCard
        items={history.items}
        nameOf={nameOf}
        onCopy={history.copy}
        onTogglePin={history.togglePin}
        onClear={history.clear}
      />
    </>
  );
}
