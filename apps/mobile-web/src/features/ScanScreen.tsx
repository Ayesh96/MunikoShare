import { messages } from "../messages/app";
import { Card } from "../components/ui";
import styles from "./Screen.module.css";

export function ScanScreen() {
  return (
    <main className={styles.page}>
      <Card title={messages.scan.title}>
        <p className={styles.paragraph}>{messages.scan.body}</p>
      </Card>
    </main>
  );
}
