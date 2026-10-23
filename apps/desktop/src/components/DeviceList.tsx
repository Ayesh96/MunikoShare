import { NEARBY_EMPTY_ICON, SEND_CLIPBOARD_ICON, SEND_FILES_ICON, SEND_FOLDER_ICON } from "../constants/icons";
import { ADD_BY_IP_LABEL, AUTO_ACCEPT_LABEL, CLIPBOARD_SYNC_HINT, CLIPBOARD_SYNC_LABEL, NEARBY_EMPTY_HINT, NEARBY_EMPTY_TITLE, NEARBY_TITLE, PAIRED_BADGE, PAIR_LABEL, SEND_CLIPBOARD_LABEL, SEND_FILES_LABEL, SEND_FOLDER_LABEL, UNPAIR_LABEL } from "../messages/ui";
import type { DeviceInfo } from "../types/device";
import styles from "./DeviceList.module.css";
import { Badge, Button, Card, EmptyState, List, ListItem, Skeleton, Switch } from "./ui";

export interface DeviceListProps {
  peers: DeviceInfo[];
  loading: boolean;
  onAddDevice: () => void;
  trusted: Set<string>;
  autoAccept: Set<string>;
  clipboardSync: Set<string>;
  onSendFiles: (peer: DeviceInfo) => void;
  onSendFolder: (peer: DeviceInfo) => void;
  onSendClipboard: (peer: DeviceInfo) => void;
  onToggleAutoAccept: (peer: DeviceInfo, enabled: boolean) => void;
  onToggleClipboardSync: (peer: DeviceInfo, enabled: boolean) => void;
  onPair: (peer: DeviceInfo) => void;
  onUnpair: (peer: DeviceInfo) => void;
}

export function DeviceList(props: DeviceListProps) {
  const { peers, loading, onAddDevice, trusted, autoAccept, clipboardSync, onSendFiles, onSendFolder, onSendClipboard, onToggleAutoAccept, onToggleClipboardSync, onPair, onUnpair } = props;
  const hasPaired = peers.some((peer) => trusted.has(peer.id));
  return (
    <Card title={NEARBY_TITLE}>
      {loading ? (
        <Skeleton />
      ) : peers.length === 0 ? (
        <EmptyState icon={NEARBY_EMPTY_ICON} title={NEARBY_EMPTY_TITLE} hint={NEARBY_EMPTY_HINT} actionLabel={ADD_BY_IP_LABEL} onAction={onAddDevice} />
      ) : (
        <List>
          {peers.map((peer) => (
            <ListItem
              key={peer.id}
              title={peer.name}
              subtitle={peer.os}
              trailing={
                <div className={styles.actions}>
                  {trusted.has(peer.id) ? (
                    <>
                      <Badge tone="success" label={PAIRED_BADGE} />
                      <Switch label={AUTO_ACCEPT_LABEL} checked={autoAccept.has(peer.id)} onChange={(enabled) => onToggleAutoAccept(peer, enabled)} />
                      <Switch label={CLIPBOARD_SYNC_LABEL} checked={clipboardSync.has(peer.id)} onChange={(enabled) => onToggleClipboardSync(peer, enabled)} />
                      <Button size="sm" variant="secondary" icon={SEND_FILES_ICON} label={SEND_FILES_LABEL} onClick={() => onSendFiles(peer)} />
                      <Button size="sm" variant="secondary" icon={SEND_FOLDER_ICON} label={SEND_FOLDER_LABEL} onClick={() => onSendFolder(peer)} />
                      <Button size="sm" variant="secondary" icon={SEND_CLIPBOARD_ICON} label={SEND_CLIPBOARD_LABEL} onClick={() => onSendClipboard(peer)} />
                      <Button size="sm" variant="secondary" label={UNPAIR_LABEL} onClick={() => onUnpair(peer)} />
                    </>
                  ) : (
                    <Button size="sm" label={PAIR_LABEL} onClick={() => onPair(peer)} />
                  )}
                </div>
              }
            />
          ))}
        </List>
      )}
      {hasPaired && <p className={styles.hint}>{CLIPBOARD_SYNC_HINT}</p>}
    </Card>
  );
}
