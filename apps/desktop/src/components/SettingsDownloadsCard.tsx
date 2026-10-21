import {
  SETTINGS_CHOOSE_FOLDER_LABEL,
  SETTINGS_DOWNLOADS_HINT,
  SETTINGS_DOWNLOADS_TITLE,
  SETTINGS_OPEN_FOLDER_LABEL,
  SETTINGS_RESET_FOLDER_LABEL,
} from "../messages/settings";
import styles from "./SettingsTab.module.css";
import { Button, Card } from "./ui";

export interface SettingsDownloadsCardProps {
  folder: string;
  onChoose: () => void;
  onReset: () => void;
  onOpen: () => void;
}

export function SettingsDownloadsCard({ folder, onChoose, onReset, onOpen }: SettingsDownloadsCardProps) {
  return (
    <Card title={SETTINGS_DOWNLOADS_TITLE}>
      <div className={styles.path}>{folder}</div>
      <div className={styles.hint}>{SETTINGS_DOWNLOADS_HINT}</div>
      <div className={styles.actions}>
        <Button label={SETTINGS_CHOOSE_FOLDER_LABEL} onClick={onChoose} />
        <Button variant="secondary" label={SETTINGS_RESET_FOLDER_LABEL} onClick={onReset} />
        <Button variant="secondary" label={SETTINGS_OPEN_FOLDER_LABEL} onClick={onOpen} />
      </div>
    </Card>
  );
}
