import { Button, Card, EmptyState, List, ListItem } from "../components/ui";
import { api } from "../lib/api";
import { formatBytes, formatRelativeTime } from "../lib/format";
import { messages } from "../messages/app";
import type { InboxItem } from "../types/api";

export function Inbox({ items }: { items: InboxItem[] }) {
  return (
    <Card title={messages.inbox.title}>
      {items.length === 0 ? (
        <EmptyState message={messages.inbox.empty} />
      ) : (
        <List>
          {items.map((item) => (
            <ListItem
              key={item.id}
              title={item.name}
              subtitle={`${formatBytes(item.size)} · ${formatRelativeTime(item.addedAt)}`}
              trailing={<Button label={messages.inbox.download} size="sm" variant="secondary" href={api.inboxUrl(item.id)} />}
            />
          ))}
        </List>
      )}
    </Card>
  );
}
