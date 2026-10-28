import { useState, type ReactNode } from "react";

import { BOARD_TEXT_COLLAPSE_LENGTH, PASTED_IMAGE_PREFIX } from "../constants/board";
import type { IconName } from "../constants/icons";
import { canPreviewImage, isImageItem } from "../lib/boardItems";
import { formatBytes } from "../lib/format";
import { formatTimestamp } from "../lib/time";
import { BOARD_AUTHOR_OFFLINE, BOARD_COPY_LABEL, BOARD_DELETE_LABEL, BOARD_DOWNLOAD_LABEL, BOARD_IMAGE_TITLE, BOARD_SHOW_LESS_LABEL, BOARD_SHOW_MORE_LABEL, BOARD_YOU, boardFileRow, boardItemRow } from "../messages/board";
import type { BoardItem } from "../types/board";
import { BoardImageThumbnail } from "./BoardImageThumbnail";
import styles from "./BoardItemList.module.css";
import { Button, Card, EmptyState, List, ListItem } from "./ui";

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

interface ItemActionsProps extends HandlerProps {
  item: BoardItem;
  expanded: boolean;
  onToggleExpanded: (item: BoardItem) => void;
}

function isLongText(item: BoardItem): boolean {
  return item.kind.type === "text" && item.kind.text.length > BOARD_TEXT_COLLAPSE_LENGTH;
}

function isUnavailable(item: BoardItem): boolean {
  return item.kind.type === "file" && !item.available && !item.heldLocally;
}

function ItemActions({ item, expanded, onToggleExpanded, onCopy, onDownload, onDelete }: ItemActionsProps) {
  const image = isImageItem(item);
  const copies = item.kind.type === "text" || image;
  const disabled = isUnavailable(item);
  return (
    <div className={styles.actions}>
      {copies ? (
        <Button size="sm" label={BOARD_COPY_LABEL} disabled={disabled} onClick={() => onCopy(item)} />
      ) : (
        <Button size="sm" label={BOARD_DOWNLOAD_LABEL} disabled={disabled} onClick={() => onDownload(item)} />
      )}
      {isLongText(item) && (
        <Button
          size="sm"
          variant="tertiary"
          iconOnly
          icon={expanded ? "collapse" : "expand"}
          label={expanded ? BOARD_SHOW_LESS_LABEL : BOARD_SHOW_MORE_LABEL}
          onClick={() => onToggleExpanded(item)}
        />
      )}
      {image && <Button size="sm" variant="tertiary" iconOnly icon="download" label={BOARD_DOWNLOAD_LABEL} disabled={disabled} onClick={() => onDownload(item)} />}
      {item.mine && <Button size="sm" variant="tertiary" iconOnly icon="trash" label={BOARD_DELETE_LABEL} onClick={() => onDelete(item)} />}
    </div>
  );
}

function itemDetails(item: BoardItem): ReactNode {
  const preview = canPreviewImage(item);
  const offline = isUnavailable(item);
  if (!preview && !offline) return null;
  return (
    <div className={styles.details}>
      {preview && <BoardImageThumbnail itemId={item.id} />}
      {offline && <span className={styles.note}>{BOARD_AUTHOR_OFFLINE}</span>}
    </div>
  );
}

function collapsed(title: string, item: BoardItem, expanded: boolean): string {
  return isLongText(item) && !expanded ? `${title.slice(0, BOARD_TEXT_COLLAPSE_LENGTH)}…` : title;
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
        <List>
          {items.map((item) => {
            const described = describe(item);
            const expanded = expandedIds.has(item.id);
            return (
              <ListItem
                key={item.id}
                title={collapsed(described.title, item, expanded)}
                subtitle={described.subtitle}
                wrap={item.kind.type === "text"}
                details={itemDetails(item)}
                trailing={
                  <ItemActions item={item} expanded={expanded} onToggleExpanded={toggle} onCopy={onCopy} onDownload={onDownload} onDelete={onDelete} />
                }
              />
            );
          })}
        </List>
      )}
    </Card>
  );
}
