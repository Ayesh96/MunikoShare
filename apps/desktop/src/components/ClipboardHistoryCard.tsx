import { CLIPBOARD_CLEAR_ICON, CLIPBOARD_COPY_ICON, CLIPBOARD_EMPTY_ICON, CLIPBOARD_PIN_ICON, CLIPBOARD_TEXT_ROW_ICON, CLIPBOARD_UNPIN_ICON } from "../constants/icons";
import { formatBytes } from "../lib/format";
import { formatTimestamp } from "../lib/time";
import { CLEAR_CLIPBOARD_LABEL, CLIPBOARD_EMPTY_HINT, CLIPBOARD_EMPTY_TITLE, CLIPBOARD_HISTORY_TITLE, clipboardImageTitle, clipboardItemRow, clipboardTextLength, COPY_LABEL, PIN_LABEL, SOURCE_THIS_DEVICE, UNPIN_LABEL } from "../messages/ui";
import type { ClipboardItem } from "../types/clipboard";
import { ClipboardThumbnail } from "./ClipboardThumbnail";
import { Button, Card, EmptyState, Icon } from "./ui";
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
          <ul className={styles.list}>
            {items.map((item) => {
              const { title, detail } = describe(item);
              const source = item.source.type === "local" ? SOURCE_THIS_DEVICE : nameOf(item.source.deviceId);
              return (
                <li key={item.id} className={styles.row}>
                  <span className={styles.tile}>
                    {item.preview.type === "image" ? <ClipboardThumbnail id={item.id} /> : <Icon name={CLIPBOARD_TEXT_ROW_ICON} />}
                  </span>
                  <span className={styles.text}>
                    <span className={styles.title}>{title}</span>
                    <span className={styles.subtitle}>{clipboardItemRow(detail, source, formatTimestamp(item.timestamp))}</span>
                  </span>
                  <span className={styles.actions}>
                    <Button iconOnly size="sm" variant="tertiary" icon={CLIPBOARD_COPY_ICON} label={COPY_LABEL} onClick={() => onCopy(item)} />
                    <Button
                      iconOnly
                      size="sm"
                      variant="tertiary"
                      icon={item.pinned ? CLIPBOARD_UNPIN_ICON : CLIPBOARD_PIN_ICON}
                      label={item.pinned ? UNPIN_LABEL : PIN_LABEL}
                      onClick={() => onTogglePin(item)}
                    />
                  </span>
                </li>
              );
            })}
          </ul>
          <Button size="sm" variant="secondary" icon={CLIPBOARD_CLEAR_ICON} label={CLEAR_CLIPBOARD_LABEL} onClick={onClear} />
        </>
      )}
    </Card>
  );
}
