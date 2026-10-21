// Mirrors muniko_core::DeviceInfo (serde default: snake_case fields).
export interface DeviceInfo {
  id: string;
  name: string;
  os: string;
  addresses: string[];
  port: number;
  protocol_version: number;
}
