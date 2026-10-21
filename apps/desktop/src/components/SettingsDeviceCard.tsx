import { useState } from "react";

import { DEVICE_NAME_MAX_LENGTH } from "../constants/settings";
import {
  SETTINGS_DEVICE_NAME_HINT,
  SETTINGS_DEVICE_NAME_LABEL,
  SETTINGS_DEVICE_TITLE,
  SETTINGS_SAVE_LABEL,
} from "../messages/settings";
import styles from "./SettingsTab.module.css";
import { Button, Card, TextInput } from "./ui";

export interface SettingsDeviceCardProps {
  savedName: string;
  onSave: (name: string) => void;
}

export function SettingsDeviceCard({ savedName, onSave }: SettingsDeviceCardProps) {
  const [name, setName] = useState(savedName);
  return (
    <Card title={SETTINGS_DEVICE_TITLE}>
      <TextInput value={name} onChange={setName} label={SETTINGS_DEVICE_NAME_LABEL} maxLength={DEVICE_NAME_MAX_LENGTH} />
      <div className={styles.hint}>{SETTINGS_DEVICE_NAME_HINT}</div>
      <Button label={SETTINGS_SAVE_LABEL} onClick={() => onSave(name)} disabled={name.trim() === savedName} />
    </Card>
  );
}
