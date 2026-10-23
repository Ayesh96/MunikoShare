import { SEND_TO_EMPTY_ICON } from "../constants/icons";
import type { PhoneSession } from "../types/phone";
import type { DeviceInfo } from "../types/device";
import { SEND_TO_CANCEL_LABEL, SEND_TO_DEVICES_HEADING, SEND_TO_EMPTY_HINT, SEND_TO_EMPTY_TITLE, SEND_TO_PHONES_HEADING, SEND_TO_SEND_LABEL, SEND_TO_TITLE, sendToSummary } from "../messages/settings";
import { fileName } from "../lib/format";
import { Button, EmptyState, List, ListItem, Modal } from "./ui";

export interface SendToModalProps {
  paths: string[];
  devices: DeviceInfo[];
  phones: PhoneSession[];
  onSendDevice: (device: DeviceInfo) => void;
  onSendPhone: (phone: PhoneSession) => void;
  onClose: () => void;
}

/** Picks the target for files passed on the command line or from the OS "Send to" menu. */
export function SendToModal({ paths, devices, phones, onSendDevice, onSendPhone, onClose }: SendToModalProps) {
  const empty = devices.length === 0 && phones.length === 0;
  const summary = paths.length === 1 ? fileName(paths[0]) : sendToSummary(paths.length);
  return (
    <Modal open title={SEND_TO_TITLE} onClose={onClose} actions={[{ label: SEND_TO_CANCEL_LABEL, onClick: onClose }]}>
      <p>{summary}</p>
      {empty && <EmptyState icon={SEND_TO_EMPTY_ICON} title={SEND_TO_EMPTY_TITLE} hint={SEND_TO_EMPTY_HINT} />}
      {devices.length > 0 && (
        <>
          <h3>{SEND_TO_DEVICES_HEADING}</h3>
          <List>
            {devices.map((device) => (
              <ListItem
                key={device.id}
                title={device.name}
                subtitle={device.os}
                trailing={<Button size="sm" label={SEND_TO_SEND_LABEL} onClick={() => onSendDevice(device)} />}
              />
            ))}
          </List>
        </>
      )}
      {phones.length > 0 && (
        <>
          <h3>{SEND_TO_PHONES_HEADING}</h3>
          <List>
            {phones.map((phone) => (
              <ListItem
                key={phone.id}
                title={phone.label}
                trailing={<Button size="sm" label={SEND_TO_SEND_LABEL} onClick={() => onSendPhone(phone)} />}
              />
            ))}
          </List>
        </>
      )}
    </Modal>
  );
}
