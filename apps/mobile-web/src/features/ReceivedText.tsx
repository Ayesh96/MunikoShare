import { Banner, Button, Card, EmptyState } from "../components/ui";
import { useCopied } from "../hooks/useCopied";
import { formatRelativeTime } from "../lib/format";
import { messages } from "../messages/app";
import type { SharedText } from "../types/api";
import styles from "./Screen.module.css";

export function ReceivedText({ shared }: { shared: SharedText | null }) {
  const { copied, copy } = useCopied(shared?.text ?? "");

  return (
    <Card title={messages.text.receiveTitle}>
      {shared ? (
        <>
          <p className={styles.received}>{shared.text}</p>
          <Banner message={formatRelativeTime(shared.sentAt)} />
          <Button label={copied ? messages.text.copied : messages.text.copy} fullWidth onClick={copy} />
        </>
      ) : (
        <EmptyState message={messages.text.receiveEmpty} />
      )}
      <Banner message={messages.text.clipboardNote} />
    </Card>
  );
}
