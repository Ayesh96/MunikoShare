import { Card, EmptyState, FilePicker } from "../components/ui";
import { useUploads } from "../hooks/useUploads";
import { messages } from "../messages/app";
import { UploadList } from "./UploadList";

export function SendFiles() {
  const { entries, add, retry } = useUploads();
  return (
    <Card title={messages.send.title}>
      <FilePicker label={messages.send.choose} onPick={add} />
      {entries.length === 0 ? <EmptyState message={messages.send.empty} /> : <UploadList entries={entries} onRetry={retry} />}
    </Card>
  );
}
