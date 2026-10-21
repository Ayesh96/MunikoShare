// Mirrors muniko_core::trust::TrustedDevice (paired_at is unix seconds).
export interface TrustedDevice {
  id: string;
  name: string;
  os: string;
  paired_at: number;
}
