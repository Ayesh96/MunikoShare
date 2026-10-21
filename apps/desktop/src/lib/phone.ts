import { PHONE_DEVICE_PREFIX } from "../constants/board";

export const phoneDeviceId = (sessionId: string) => `${PHONE_DEVICE_PREFIX}${sessionId}`;
export const isPhoneDevice = (deviceId: string) => deviceId.startsWith(PHONE_DEVICE_PREFIX);

/** "m:ss" for a countdown; never negative. */
export function formatCountdown(totalSeconds: number): string {
  const seconds = Math.max(0, totalSeconds);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}
