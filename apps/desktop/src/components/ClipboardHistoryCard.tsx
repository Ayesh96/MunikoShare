import { CLIPBOARD_EMPTY_ICON } from "../constants/icons";
import { formatBytes } from "../lib/format";
import { formatTimestamp } from "../lib/time";
import { CLEAR_CLIPBOARD_LABEL, CLIPBOARD_EMPTY_HINT, CLIPBOARD_EMPTY_TITLE, CLIPBOARD_HISTORY_TITLE, clipboardImageTitle, clipboardItemRow, clipboardTextLength, COPY_LABEL, PIN_LABEL, PINNED_LABEL, SOURCE_THIS_DEVICE, UNPIN_LABEL } from "../messages/ui";
import type { ClipboardItem } from "../types/clipboard";
import { ClipboardThumbnail } from "./ClipboardThumbnail";
import { Badge, Button, Card, EmptyState, List, ListItem } from "./ui";
import styles from "./ClipboardHistoryCard.module.css";

export interface ClipboardHistoryCardProps {
  items: ClipboardItem[];
  nameOf: (deviceId: string) => string;
  onCopy: (item: ClipboardItem) => void;
  onTogglePin: (item: ClipboardItem) => void;
  onClear: () => void;
}

function describe(item: ClipboardItem): { title: string; detail: string } {
  const { preview } = item;
  if (preview.type === "text") return { title: preview.preview, detail: clipboardTextLength(preview.length) };
  return { title: clipboardImageTitle(preview.width, preview.height), detail: formatBytes(preview.bytes) };
}

export function ClipboardHistoryCard({ items, nameOf, onCopy, onTogglePin, onClear }: ClipboardHistoryCardProps) {
  return (
    <Card title={CLIPBOARD_HISTORY_TITLE}>
      {items.length === 0 ? (
        <EmptyState icon={CLIPBOARD_EMPTY_ICON} title={CLIPBOARD_EMPTY_TITLE} hint={CLIPBOARD_EMPTY_HINT} />
      ) : (
        <>
          <List>
            {items.map((item) => {
              const { title, detail } = describe(item);
              const source = item.source.type === "local" ? SOURCE_THIS_DEVICE : nameOf(item.source.deviceId);
              return (
                <ListItem
                  key={item.id}
                  title={title}
                  subtitle={clipboardItemRow(detail, source, formatTimestamp(item.timestamp))}
                  leading={
                    item.preview.type === "image" ? (
                      <ClipboardThumbnail id={item.id} width={item.preview.width} height={item.preview.height} />
                    ) : undefined
                  }
                  trailing={
                    <div className={styles.actions}>
                      {item.pinned && <Badge label={PINNED_LABEL} />}
                      <Button size="sm" label={COPY_LABEL} onClick={() => onCopy(item)} />
                      <Button size="sm" variant="secondary" label={item.pinned ? UNPIN_LABEL : PIN_LABEL} onClick={() => onTogglePin(item)} />
                    </div>
                  }
                />
              );
            })}
          </List>
          <Button size="sm" variant="secondary" label={CLEAR_CLIPBOARD_LABEL} onClick={onClear} />
        </>
      )}
    </Card>
  );
}
