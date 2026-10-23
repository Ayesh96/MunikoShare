import { PHONE_EMPTY_ICON } from "../constants/icons";
import { formatTimestamp } from "../lib/time";
import { PHONE_DISCONNECT_LABEL, PHONE_EMPTY_HINT, PHONE_EMPTY_TITLE, PHONE_LIST_TITLE, PHONE_SEND_FILES_LABEL, PHONE_SEND_TEXT_LABEL, phoneRow } from "../messages/phone";
import type { PhoneSession } from "../types/phone";
import styles from "./PhoneList.module.css";
import { Button, Card, EmptyState, List, ListItem } from "./ui";

export interface PhoneListProps {
  sessions: PhoneSession[];
  onSendFiles: (session: PhoneSession) => void;
  onSendText: (session: PhoneSession) => void;
  onDisconnect: (session: PhoneSession) => void;
}

export function PhoneList({ sessions, onSendFiles, onSendText, onDisconnect }: PhoneListProps) {
  return (
    <Card title={PHONE_LIST_TITLE}>
      {sessions.length === 0 ? (
        <EmptyState icon={PHONE_EMPTY_ICON} title={PHONE_EMPTY_TITLE} hint={PHONE_EMPTY_HINT} />
      ) : (
        <List>
          {sessions.map((session) => (
            <ListItem
              key={session.id}
              title={session.label}
              subtitle={phoneRow(formatTimestamp(session.connectedAt), formatTimestamp(session.lastSeen))}
              trailing={
                <div className={styles.actions}>
                  <Button size="sm" label={PHONE_SEND_FILES_LABEL} onClick={() => onSendFiles(session)} />
                  <Button size="sm" variant="secondary" label={PHONE_SEND_TEXT_LABEL} onClick={() => onSendText(session)} />
                  <Button size="sm" variant="secondary" label={PHONE_DISCONNECT_LABEL} onClick={() => onDisconnect(session)} />
                </div>
              }
            />
          ))}
        </List>
      )}
    </Card>
  );
}
