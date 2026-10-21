// Mirrors muniko_core::mobile (serde camelCase; timestamps in unix seconds).
export interface PhoneSession {
  id: string;
  label: string;
  connectedAt: number;
  lastSeen: number;
}

export interface PhoneInvite {
  urls: string[];
  token: string;
  expiresAt: number;
}
