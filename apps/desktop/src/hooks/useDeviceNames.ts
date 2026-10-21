import { useCallback, useRef } from "react";

import { isPhoneDevice, phoneDeviceId } from "../lib/phone";
import { shortId } from "../lib/format";
import { PHONE_NAME, phoneName } from "../messages/phone";
import type { DeviceInfo } from "../types/device";
import type { PhoneSession } from "../types/phone";
import type { TrustedDevice } from "../types/trust";

/** Remembers names of every peer, paired device and phone seen; unknown ids fall back to a short id. */
export function useDeviceNames(peers: DeviceInfo[], trusted: TrustedDevice[], phones: PhoneSession[]): (deviceId: string) => string {
  const names = useRef(new Map<string, string>());
  trusted.forEach((device) => names.current.set(device.id, device.name));
  peers.forEach((peer) => names.current.set(peer.id, peer.name));
  phones.forEach((phone) => names.current.set(phoneDeviceId(phone.id), phoneName(phone.label)));
  return useCallback((deviceId: string) => names.current.get(deviceId) ?? (isPhoneDevice(deviceId) ? PHONE_NAME : shortId(deviceId)), []);
}
