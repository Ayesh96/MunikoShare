import { useState } from "react";

import { DEFAULT_PORT } from "../constants/app";
import { errorMessage, parsePort } from "../lib/format";
import {
  ADD_DEVICE_CONFIRM_LABEL,
  ADD_DEVICE_INVALID_PORT,
  ADD_DEVICE_IP_LABEL,
  ADD_DEVICE_IP_PLACEHOLDER,
  ADD_DEVICE_PORT_LABEL,
  ADD_DEVICE_TITLE,
  PAIR_CANCEL_LABEL,
} from "../messages/ui";
import { Banner, Modal, TextInput } from "./ui";

export interface AddDeviceModalProps {
  onAdd: (ip: string, port: number) => Promise<void>;
  onClose: () => void;
}

/** Adds a device that discovery cannot see, by its IP address and port. */
export function AddDeviceModal({ onAdd, onClose }: AddDeviceModalProps) {
  const [ip, setIp] = useState("");
  const [port, setPort] = useState(String(DEFAULT_PORT));
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  async function submit() {
    const parsed = parsePort(port);
    if (parsed === null) {
      setError(ADD_DEVICE_INVALID_PORT);
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      await onAdd(ip.trim(), parsed);
    } catch (failure) {
      setError(errorMessage(failure));
      setBusy(false);
    }
  }

  return (
    <Modal
      open
      title={ADD_DEVICE_TITLE}
      onClose={onClose}
      actions={[
        { label: PAIR_CANCEL_LABEL, onClick: onClose },
        { label: ADD_DEVICE_CONFIRM_LABEL, onClick: submit, variant: "primary", disabled: busy || ip.trim() === "" },
      ]}
    >
      {error && <Banner message={error} tone="danger" />}
      <TextInput value={ip} onChange={setIp} label={ADD_DEVICE_IP_LABEL} placeholder={ADD_DEVICE_IP_PLACEHOLDER} autoFocus />
      <TextInput value={port} onChange={setPort} label={ADD_DEVICE_PORT_LABEL} inputMode="numeric" maxLength={5} />
    </Modal>
  );
}
