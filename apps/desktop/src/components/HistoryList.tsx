import { HISTORY_EMPTY_ICON } from "../constants/icons";
import { HISTORY_BADGES, historyTitle } from "../lib/history";
import { formatTimestamp } from "../lib/time";
import { CLEAR_HISTORY_LABEL, HISTORY_EMPTY_HINT, HISTORY_EMPTY_TITLE, HISTORY_TITLE, historyRow } from "../messages/ui";
import type { HistoryEntry } from "../types/history";
import { Badge, Button, Card, EmptyState, List, ListItem } from "./ui";

export interface HistoryListProps {
  entries: HistoryEntry[];
  nameOf: (deviceId: string) => string;
  onClear: () => void;
}

export function HistoryList({ entries, nameOf, onClear }: HistoryListProps) {
  return (
    <Card title={HISTORY_TITLE}>
      {entries.length === 0 ? (
        <EmptyState icon={HISTORY_EMPTY_ICON} title={HISTORY_EMPTY_TITLE} hint={HISTORY_EMPTY_HINT} />
      ) : (
        <>
          <List>
            {entries.map((entry) => {
              const badge = HISTORY_BADGES[entry.kind];
              return (
                <ListItem
                  key={entry.id}
                  title={historyTitle(entry)}
                  subtitle={historyRow(nameOf(entry.deviceId), formatTimestamp(entry.timestamp))}
                  leading={<Badge tone={badge.tone} label={badge.label} />}
                />
              );
            })}
          </List>
          <Button size="sm" variant="secondary" label={CLEAR_HISTORY_LABEL} onClick={onClear} />
        </>
      )}
    </Card>
  );
}
