import { localAddresses } from "../lib/tauriApi";
import { useLoaded } from "./useLoaded";

/** LAN IPv4 addresses of this machine. */
export function useLocalAddresses(): string[] {
  return useLoaded(localAddresses).value ?? [];
}
