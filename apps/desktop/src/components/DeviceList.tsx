import { useState } from "react";

import { DEVICE_OPTIONS_ICON, NEARBY_EMPTY_ICON, SEND_CLIPBOARD_ICON, SEND_FILES_ICON, SEND_FOLDER_ICON } from "../constants/icons";
import { ADD_BY_IP_LABEL, AUTO_ACCEPT_LABEL, CLIPBOARD_SYNC_HINT, CLIPBOARD_SYNC_LABEL, DEVICE_OPTIONS_HIDE_LABEL, DEVICE_OPTIONS_LABEL, DEVICES_DROP_HINT, NEARBY_EMPTY_HINT, NEARBY_EMPTY_TITLE, NEARBY_TITLE, PAIR_TO_SEND_LABEL, SEND_CLIPBOARD_LABEL, SEND_FILES_LABEL, SEND_FOLDER_LABEL, UNPAIR_LABEL, unpairedSubtitle } from "../messages/ui";
import type { DeviceInfo } from "../types/device";
import styles from "./DeviceList.module.css";
import { Button, Card, EmptyState, List, ListItem, Skeleton, Switch } from "./ui";

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

interface PeerProps extends DeviceListProps {
  peer: DeviceInfo;
  expanded: boolean;
  onToggleExpanded: () => void;
}

function SendActions(props: PeerProps) {
  const { peer, expanded, onSendFiles, onSendFolder, onSendClipboard } = props;
  return (
    <div className={styles.details}>
      <div className={styles.actions}>
        <Button size="sm" icon={SEND_FILES_ICON} label={SEND_FILES_LABEL} onClick={() => onSendFiles(peer)} />
        <Button size="sm" variant="secondary" icon={SEND_FOLDER_ICON} label={SEND_FOLDER_LABEL} onClick={() => onSendFolder(peer)} />
        <Button size="sm" variant="secondary" icon={SEND_CLIPBOARD_ICON} label={SEND_CLIPBOARD_LABEL} onClick={() => onSendClipboard(peer)} />
      </div>
      {expanded && <PeerOptions {...props} />}
    </div>
  );
}

function PeerOptions({ peer, autoAccept, clipboardSync, onToggleAutoAccept, onToggleClipboardSync, onUnpair }: PeerProps) {
  return (
    <div className={styles.options}>
      <Switch label={AUTO_ACCEPT_LABEL} checked={autoAccept.has(peer.id)} onChange={(enabled) => onToggleAutoAccept(peer, enabled)} />
      <Switch label={CLIPBOARD_SYNC_LABEL} checked={clipboardSync.has(peer.id)} onChange={(enabled) => onToggleClipboardSync(peer, enabled)} />
      <p className={styles.hint}>{CLIPBOARD_SYNC_HINT}</p>
      <div>
        <Button size="sm" variant="destructive" label={UNPAIR_LABEL} onClick={() => onUnpair(peer)} />
      </div>
    </div>
  );
}

function PeerRow(props: PeerProps) {
  const { peer, trusted, expanded, onPair } = props;
  if (!trusted.has(peer.id)) {
    return (
      <ListItem
        title={peer.name}
        subtitle={unpairedSubtitle(peer.os)}
        trailing={<Button size="sm" label={PAIR_TO_SEND_LABEL} onClick={() => onPair(peer)} />}
      />
    );
  }
  return (
    <ListItem
      title={peer.name}
      subtitle={peer.os}
      trailing={
        <Button size="sm" variant="tertiary" icon={DEVICE_OPTIONS_ICON} label={expanded ? DEVICE_OPTIONS_HIDE_LABEL : DEVICE_OPTIONS_LABEL} onClick={props.onToggleExpanded} />
      }
      details={<SendActions {...props} />}
    />
  );
}

export function DeviceList(props: DeviceListProps) {
  const { peers, loading, onAddDevice, trusted } = props;
  const [expandedId, setExpandedId] = useState<string | null>(null);
  // Paired devices first, since those are the ones you can share with right away.
  const sorted = [...peers].sort((a, b) => Number(trusted.has(b.id)) - Number(trusted.has(a.id)));
  const hasPaired = peers.some((peer) => trusted.has(peer.id));
  return (
    <Card title={NEARBY_TITLE}>
      {loading ? (
        <Skeleton />
      ) : peers.length === 0 ? (
        <EmptyState icon={NEARBY_EMPTY_ICON} title={NEARBY_EMPTY_TITLE} hint={NEARBY_EMPTY_HINT} actionLabel={ADD_BY_IP_LABEL} onAction={onAddDevice} />
      ) : (
        <List>
          {sorted.map((peer) => (
            <PeerRow
              key={peer.id}
              {...props}
              peer={peer}
              expanded={expandedId === peer.id}
              onToggleExpanded={() => setExpandedId((current) => (current === peer.id ? null : peer.id))}
            />
          ))}
        </List>
      )}
      {hasPaired && <p className={styles.hint}>{DEVICES_DROP_HINT}</p>}
    </Card>
  );
}
