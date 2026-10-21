import { getLocalDevice } from "../lib/tauriApi";
import type { DeviceInfo } from "../types/device";
import { useLoaded } from "./useLoaded";

export function useLocalDevice(): DeviceInfo | null {
  return useLoaded(getLocalDevice).value;
}
