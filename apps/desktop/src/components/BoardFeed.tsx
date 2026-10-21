import { useState } from "react";

import { BOARD_TEXT_COLLAPSE_LENGTH } from "../constants/board";
import { formatBytes } from "../lib/format";
import { formatTimestamp } from "../lib/time";
import {
  BOARD_AUTHOR_OFFLINE,
  BOARD_COPY_LABEL,
  BOARD_DELETE_LABEL,
  BOARD_DOWNLOAD_LABEL,
  BOARD_FEED_EMPTY_HINT,
  BOARD_FEED_EMPTY_ICON,
  BOARD_FEED_EMPTY_TITLE,
  BOARD_FEED_TITLE,
  BOARD_SAVED_BADGE,
  BOARD_SHOW_LESS_LABEL,
  BOARD_SHOW_MORE_LABEL,
  BOARD_YOU,
  boardFileRow,
  boardItemRow,
} from "../messages/board";
import type { BoardItem } from "../types/board";
import styles from "./BoardItemList.module.css";
import { Badge, Button, Card, EmptyState, List, ListItem } from "./ui";

export interface BoardFeedProps {
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
  return { title: kind.name, subtitle: boardFileRow(formatBytes(kind.size), author, time) };
}

interface ItemActionsProps extends Omit<BoardFeedProps, "items"> {
  item: BoardItem;
  expanded: boolean;
  onToggleExpanded: (item: BoardItem) => void;
}

function isLongText(item: BoardItem): boolean {
  return item.kind.type === "text" && item.kind.text.length > BOARD_TEXT_COLLAPSE_LENGTH;
}

function ItemActions({ item, expanded, onToggleExpanded, onCopy, onDownload, onDelete }: ItemActionsProps) {
  const unavailable = item.kind.type === "file" && !item.available && !item.heldLocally;
  return (
    <div className={styles.actions}>
      {isLongText(item) && (
        <Button size="sm" variant="secondary" label={expanded ? BOARD_SHOW_LESS_LABEL : BOARD_SHOW_MORE_LABEL} onClick={() => onToggleExpanded(item)} />
      )}
      {item.kind.type === "file" && item.heldLocally && <Badge tone="success" label={BOARD_SAVED_BADGE} />}
      {item.kind.type === "text" ? (
        <Button size="sm" label={BOARD_COPY_LABEL} onClick={() => onCopy(item)} />
      ) : (
        <Button
          size="sm"
          label={unavailable ? BOARD_AUTHOR_OFFLINE : BOARD_DOWNLOAD_LABEL}
          disabled={unavailable}
          onClick={() => onDownload(item)}
        />
      )}
      {item.mine && <Button size="sm" variant="secondary" label={BOARD_DELETE_LABEL} onClick={() => onDelete(item)} />}
    </div>
  );
}

function collapsed(title: string, item: BoardItem, expanded: boolean): string {
  return isLongText(item) && !expanded ? `${title.slice(0, BOARD_TEXT_COLLAPSE_LENGTH)}…` : title;
}

export function BoardFeed({ items, onCopy, onDownload, onDelete }: BoardFeedProps) {
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const toggle = (item: BoardItem) =>
    setExpandedIds((current) => {
      const next = new Set(current);
      if (!next.delete(item.id)) next.add(item.id);
      return next;
    });
  return (
    <Card title={BOARD_FEED_TITLE}>
      {items.length === 0 ? (
        <EmptyState icon={BOARD_FEED_EMPTY_ICON} title={BOARD_FEED_EMPTY_TITLE} hint={BOARD_FEED_EMPTY_HINT} />
      ) : (
        <List>
          {items.map((item) => {
            const { title, subtitle } = describe(item);
            const expanded = expandedIds.has(item.id);
            return (
              <ListItem
                key={item.id}
                title={collapsed(title, item, expanded)}
                subtitle={subtitle}
                wrap={item.kind.type === "text"}
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
