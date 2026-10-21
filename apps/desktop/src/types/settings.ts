// Mirrors the shell's SettingsView (serde camelCase).
export interface Settings {
  deviceName: string;
  downloadDir: string;
}

export type UpdateStatus =
  | { type: "idle" }
  | { type: "checking" }
  | { type: "upToDate" }
  | { type: "available"; version: string }
  | { type: "installing" }
  | { type: "error"; message: string };
