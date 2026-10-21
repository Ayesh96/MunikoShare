import { Button, Card, EmptyState, List, ListItem, Post } from "../components/ui";
import { useCopied } from "../hooks/useCopied";
import { api } from "../lib/api";
import { formatBytes, formatRelativeTime } from "../lib/format";
import { messages } from "../messages/app";
import type { BoardItemView } from "../types/api";

function author(item: BoardItemView): string {
  return item.mine ? messages.board.you : item.authorName;
}

function TextItem({ item, text }: { item: BoardItemView; text: string }) {
  const { copied, copy } = useCopied(text);
  const label = copied ? messages.text.copied : messages.text.copy;
  return (
    <Post
      meta={messages.board.meta(author(item), formatRelativeTime(item.createdAt))}
      text={text}
      footer={<Button label={label} size="sm" variant="secondary" onClick={copy} />}
    />
  );
}

function FileItem({ item, name, size }: { item: BoardItemView; name: string; size: number }) {
  const downloadable = item.available || item.heldLocally;
  return (
    <ListItem
      title={name}
      subtitle={messages.board.fileMeta(formatBytes(size), author(item), formatRelativeTime(item.createdAt))}
      trailing={
        <Button
          label={downloadable ? messages.board.download : messages.board.authorOffline}
          size="sm"
          variant="secondary"
          href={api.boardFileUrl(item.id)}
          disabled={!downloadable}
        />
      }
    />
  );
}

export function BoardFeed({ items }: { items: BoardItemView[] }) {
  if (items.length === 0) return <Card><EmptyState message={messages.board.empty} /></Card>;
  return (
    <Card>
      <List>
        {items.map((item) =>
          item.kind.type === "text"
            ? <TextItem key={item.id} item={item} text={item.kind.text} />
            : <FileItem key={item.id} item={item} name={item.kind.name} size={item.kind.size} />,
        )}
      </List>
    </Card>
  );
}
