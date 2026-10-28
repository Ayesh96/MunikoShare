import { useState } from "react";

import { BOARD_TEXT_COLLAPSE_LENGTH, PASTED_IMAGE_PREFIX } from "../constants/board";
import { ICON_TILE_SIZE, type IconName } from "../constants/icons";
import { canPreviewImage, isImageItem } from "../lib/boardItems";
import { formatBytes } from "../lib/format";
import { formatTimestamp } from "../lib/time";
import { BOARD_AUTHOR_OFFLINE, BOARD_COPY_LABEL, BOARD_DELETE_LABEL, BOARD_DOWNLOAD_LABEL, BOARD_IMAGE_TITLE, BOARD_SHOW_LESS_LABEL, BOARD_SHOW_MORE_LABEL, BOARD_YOU, boardFileRow, boardItemRow } from "../messages/board";
import type { BoardItem } from "../types/board";
import { BoardImageThumbnail } from "./BoardImageThumbnail";
import styles from "./BoardItemList.module.css";
import { Button, Card, EmptyState, Icon } from "./ui";

export interface BoardFeedProps {
  title: string;
  emptyTitle: string;
  emptyHint: string;
  emptyIcon: IconName;
  items: BoardItem[];
  onCopy: (item: BoardItem) => void;
  onDownload: (item: BoardItem) => void;
  onDelete: (item: BoardItem) => void;
}

function describe(item: BoardItem): { title: string; subtitle: string } {
  const author = item.mine ? BOARD_YOU : item.authorName;
  const time = formatTimestamp(item.createdAt);
  const { kind } = item;
  if (kind.type === "text") return { title: kind.text, subtitle: boardItemRow(author, time) };
  const subtitle = boardFileRow(formatBytes(kind.size), author, time);
  const pasted = isImageItem(item) && kind.name.startsWith(PASTED_IMAGE_PREFIX);
  return { title: pasted ? BOARD_IMAGE_TITLE : kind.name, subtitle };
}

type HandlerProps = Pick<BoardFeedProps, "onCopy" | "onDownload" | "onDelete">;

function isLongText(item: BoardItem): boolean {
  return item.kind.type === "text" && item.kind.text.length > BOARD_TEXT_COLLAPSE_LENGTH;
}

function isUnavailable(item: BoardItem): boolean {
  return item.kind.type === "file" && !item.available && !item.heldLocally;
}

/** One icon bar per tile, pinned top-right; Copy is always last so it sits at the far right. */
function ItemActions({ item, onCopy, onDownload, onDelete }: { item: BoardItem } & HandlerProps) {
  const copies = item.kind.type === "text" || isImageItem(item);
  const downloads = item.kind.type === "file";
  const disabled = isUnavailable(item);
  return (
    <div className={styles.overlay}>
      {downloads && <Button size="sm" variant="tertiary" iconOnly icon="download" label={BOARD_DOWNLOAD_LABEL} disabled={disabled} onClick={() => onDownload(item)} />}
      {item.mine && <Button size="sm" variant="tertiary" iconOnly icon="trash" label={BOARD_DELETE_LABEL} onClick={() => onDelete(item)} />}
      {copies && <Button size="sm" variant="tertiary" iconOnly icon="copy" label={BOARD_COPY_LABEL} disabled={disabled} onClick={() => onCopy(item)} />}
    </div>
  );
}

function ItemMedia({ item }: { item: BoardItem }) {
  if (canPreviewImage(item)) {
    return (
      <div className={styles.media}>
        <BoardImageThumbnail itemId={item.id} />
      </div>
    );
  }
  const image = isImageItem(item);
  return (
    <div className={image ? styles.media : `${styles.media} ${styles.file}`}>
      <Icon name={image ? "image" : "file"} size={ICON_TILE_SIZE} />
    </div>
  );
}

function collapsed(text: string, item: BoardItem, expanded: boolean): string {
  return isLongText(item) && !expanded ? `${text.slice(0, BOARD_TEXT_COLLAPSE_LENGTH)}…` : text;
}

interface BoardTileProps extends HandlerProps {
  item: BoardItem;
  expanded: boolean;
  onToggleExpanded: (item: BoardItem) => void;
}

function BoardTile({ item, expanded, onToggleExpanded, ...handlers }: BoardTileProps) {
  const { title, subtitle } = describe(item);
  const isText = item.kind.type === "text";
  return (
    <li className={styles.tile}>
      {isText ? <div className={styles.text}>{collapsed(title, item, expanded)}</div> : <ItemMedia item={item} />}
      <ItemActions item={item} {...handlers} />
      <div className={styles.info}>
        {!isText && <span className={styles.title}>{title}</span>}
        <span className={styles.subtitle}>{subtitle}</span>
        {isLongText(item) && (
          <Button size="sm" variant="tertiary" label={expanded ? BOARD_SHOW_LESS_LABEL : BOARD_SHOW_MORE_LABEL} onClick={() => onToggleExpanded(item)} />
        )}
        {isUnavailable(item) && <span className={styles.note}>{BOARD_AUTHOR_OFFLINE}</span>}
      </div>
    </li>
  );
}

export function BoardFeed({ title, emptyTitle, emptyHint, emptyIcon, items, onCopy, onDownload, onDelete }: BoardFeedProps) {
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const toggle = (item: BoardItem) =>
    setExpandedIds((current) => {
      const next = new Set(current);
      if (!next.delete(item.id)) next.add(item.id);
      return next;
    });
  return (
    <Card title={title}>
      {items.length === 0 ? (
        <EmptyState icon={emptyIcon} title={emptyTitle} hint={emptyHint} />
      ) : (
        <ul className={styles.grid}>
          {items.map((item) => (
            <BoardTile
              key={item.id}
              item={item}
              expanded={expandedIds.has(item.id)}
              onToggleExpanded={toggle}
              onCopy={onCopy}
              onDownload={onDownload}
              onDelete={onDelete}
            />
          ))}
        </ul>
      )}
    </Card>
  );
}
