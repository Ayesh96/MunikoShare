import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";

export const autostartEnabled = () => isEnabled();
export const setAutostart = (enabled: boolean) => (enabled ? enable() : disable());
