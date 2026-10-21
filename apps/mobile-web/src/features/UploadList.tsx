import { Banner, Button, List, ListItem, ProgressBar } from "../components/ui";
import { formatBytes } from "../lib/format";
import { messages } from "../messages/app";
import type { UploadEntry, UploadStatus } from "../types/api";

const TONES = { uploading: "info", done: "success", failed: "danger" } as const;
const LABELS: Record<UploadStatus, string> = { uploading: messages.send.uploading, done: messages.send.done, failed: messages.send.failed };

export interface UploadListProps {
  entries: UploadEntry[];
  onRetry: (id: number) => void;
}

export function UploadList({ entries, onRetry }: UploadListProps) {
  return (
    <List>
      {entries.map((entry) => (
        <ListItem
          key={entry.id}
          title={entry.file.name}
          subtitle={formatBytes(entry.file.size)}
          trailing={entry.status === "failed"
            ? <Button label={messages.send.retry} size="sm" variant="secondary" onClick={() => onRetry(entry.id)} />
            : <Banner message={LABELS[entry.status]} tone={TONES[entry.status]} />}
          footer={entry.status === "done" ? undefined : <ProgressBar value={entry.progress} tone={TONES[entry.status]} />}
        />
      ))}
    </List>
  );
}
