import { useAutostart } from "../hooks/useAutostart";
import { useLoaded } from "../hooks/useLoaded";
import type { Phones } from "../hooks/usePhones";
import { useSettings } from "../hooks/useSettings";
import { useUpdater } from "../hooks/useUpdater";
import { getAppVersion } from "../lib/settingsApi";
import { PHONE_ACCESS_HINT, PHONE_ACCESS_LABEL } from "../messages/phone";
import { SETTINGS_AUTOSTART_LABEL, SETTINGS_PHONE_TITLE, SETTINGS_STARTUP_TITLE } from "../messages/settings";
import type { Tone } from "../types/ui";
import { SettingsDeviceCard } from "./SettingsDeviceCard";
import { SettingsDownloadsCard } from "./SettingsDownloadsCard";
import styles from "./SettingsTab.module.css";
import { UpdateCard } from "./UpdateCard";
import { Card, Switch } from "./ui";

export interface SettingsTabProps {
  phones: Phones;
  show: (message: string, tone: Tone) => void;
}

/** Mounted only while its tab is visible. */
export function SettingsTab({ phones, show }: SettingsTabProps) {
  const settings = useSettings(show);
  const autostart = useAutostart(show);
  const updater = useUpdater();
  const version = useLoaded(getAppVersion).value ?? "";
  const { settings: saved } = settings;

  return (
    <>
      {saved && <SettingsDeviceCard key={saved.deviceName} savedName={saved.deviceName} onSave={settings.saveName} />}
      {saved && (
        <SettingsDownloadsCard
          folder={saved.downloadDir}
          onChoose={settings.chooseFolder}
          onReset={settings.resetFolder}
          onOpen={settings.openFolder}
        />
      )}
      <Card title={SETTINGS_STARTUP_TITLE}>
        <Switch checked={autostart.enabled ?? false} disabled={autostart.enabled === null} onChange={autostart.set} label={SETTINGS_AUTOSTART_LABEL} />
      </Card>
      <Card title={SETTINGS_PHONE_TITLE}>
        <Switch checked={phones.access ?? false} disabled={phones.access === null} onChange={phones.setAccess} label={PHONE_ACCESS_LABEL} />
        <div className={styles.hint}>{PHONE_ACCESS_HINT}</div>
      </Card>
      <UpdateCard version={version} updater={updater} />
    </>
  );
}
