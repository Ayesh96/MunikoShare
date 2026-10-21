import type { DeviceInfo } from "./device";
import type { FileSummary, TransferDirection } from "./transfer";

// Mirrors muniko_core::NodeEvent: variant tag is camelCase, variant fields stay snake_case.
export type NodeEvent =
  | { type: "peerFound"; device: DeviceInfo }
  | { type: "peerLost"; device_id: string }
  | { type: "pairingRequested"; device: DeviceInfo; code: string }
  | { type: "pairingCompleted"; device_id: string; device_name: string }
  | { type: "pairingFailed"; device_id: string; reason: string }
  | { type: "deviceUnpaired"; device_id: string }
  | { type: "incomingOffer"; offer_id: string; from_device_id: string; files: FileSummary[]; total_size: number }
  | { type: "offerClosed"; offer_id: string; accepted: boolean }
  | {
      type: "transferProgress";
      transfer_id: string;
      device_id: string;
      direction: TransferDirection;
      bytes_done: number;
      bytes_total: number;
    }
  | { type: "fileReceived"; from_device_id: string; path: string }
  | { type: "fileSent"; to_device_id: string; path: string }
  | { type: "transferFailed"; device_id: string; reason: string }
  | { type: "clipboardReceived"; from_device_id: string; text: string }
  | { type: "clipboardSent"; to_device_id: string }
  | { type: "clipboardImageReceived"; from_device_id: string; width: number; height: number }
  | { type: "phoneConnected"; session_id: string; label: string }
  | { type: "phoneDisconnected"; session_id: string }
  | { type: "boardChanged"; board_id: string }
  | { type: "peerAdded"; device: DeviceInfo };
