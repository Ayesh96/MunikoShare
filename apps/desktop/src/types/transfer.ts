// Mirrors muniko_core::TransferDirection (serde camelCase).
export type TransferDirection = "incoming" | "outgoing";

// Mirrors muniko_core::FileSummary.
export interface FileSummary {
  name: string;
  size: number;
}

export interface IncomingOffer {
  offer_id: string;
  from_device_id: string;
  files: FileSummary[];
  total_size: number;
}

export interface ActiveTransfer {
  transfer_id: string;
  device_id: string;
  direction: TransferDirection;
  bytes_done: number;
  bytes_total: number;
}
