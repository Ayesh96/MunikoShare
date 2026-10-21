import type { Updater } from "../hooks/useUpdater";
import {
  SETTINGS_ABOUT_TITLE,
  SETTINGS_CHECK_UPDATES_LABEL,
  SETTINGS_INSTALL_UPDATE_LABEL,
  settingsVersion,
  UPDATE_CHECKING,
  UPDATE_INSTALLING,
  UPDATE_UP_TO_DATE,
  updateAvailable,
  updateError,
} from "../messages/settings";
import type { UpdateStatus } from "../types/settings";
import styles from "./SettingsTab.module.css";
import { Button, Card } from "./ui";

function statusText(status: UpdateStatus): string | null {
  switch (status.type) {
    case "checking": return UPDATE_CHECKING;
    case "upToDate": return UPDATE_UP_TO_DATE;
    case "available": return updateAvailable(status.version);
    case "installing": return UPDATE_INSTALLING;
    case "error": return updateError(status.message);
    default: return null;
  }
}

export function UpdateCard({ version, updater }: { version: string; updater: Updater }) {
  const { status } = updater;
  const busy = status.type === "checking" || status.type === "installing";
  const text = statusText(status);
  return (
    <Card title={SETTINGS_ABOUT_TITLE}>
      <div className={styles.path}>{settingsVersion(version)}</div>
      {text && <div className={styles.hint}>{text}</div>}
      <div className={styles.actions}>
        <Button variant="secondary" label={SETTINGS_CHECK_UPDATES_LABEL} onClick={updater.check} disabled={busy} />
        {status.type === "available" && <Button label={SETTINGS_INSTALL_UPDATE_LABEL} onClick={updater.install} />}
      </div>
    </Card>
  );
}
