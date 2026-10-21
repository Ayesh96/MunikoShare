export const NODE_EVENT_NAME = "muniko://node-event";
export const HOTKEY_EVENT_NAME = "muniko://hotkey-result";
export const LAUNCH_EVENT_NAME = "muniko://launch-paths";
export const CLIPBOARD_PREVIEW_LENGTH = 80;
export const PAIRING_CODE_LENGTH = 6;
export const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB"] as const;
export const BYTES_PER_UNIT = 1024;

/** Mirrors muniko_core DEFAULT_PORT. */
export const DEFAULT_PORT = 53420;
export const MIN_PORT = 1;
export const MAX_PORT = 65535;
export const SHORT_ID_LENGTH = 8;

export const HISTORY_LIMIT = 100;
export const HISTORY_REFRESH_EVENTS: readonly string[] = [
  "fileReceived",
  "fileSent",
  "transferFailed",
  "clipboardReceived",
  "clipboardSent",
  "pairingCompleted",
  "deviceUnpaired",
];

export const SECONDS_PER_MINUTE = 60;
export const SECONDS_PER_HOUR = 3600;
export const SECONDS_PER_DAY = 86400;
export const RELATIVE_TIME_MAX_DAYS = 7;

export const TAB_DEVICES = "devices";
export const TAB_HISTORY = "history";
export const TAB_CLIPBOARD = "clipboard";
export const TAB_BOARD = "board";
export const TAB_PHONE = "phone";
export const TAB_SETTINGS = "settings";

export const COMMANDS = {
  getLocalDevice: "get_local_device",
  listPeers: "list_peers",
  localAddresses: "local_addresses",
  addPeerByAddress: "add_peer_by_address",
  sendFiles: "send_files",
  respondToOffer: "respond_to_offer",
  requestPairing: "request_pairing",
  confirmPairing: "confirm_pairing",
  unpair: "unpair",
  listTrusted: "list_trusted",
  setAutoAccept: "set_auto_accept",
  listAutoAccept: "list_auto_accept",
  getHistory: "get_history",
  clearHistory: "clear_history",
  pushClipboard: "push_clipboard",
  pushText: "push_text",
  setClipboardSync: "set_clipboard_sync",
  listClipboardSync: "list_clipboard_sync",
  clipboardHistory: "clipboard_history",
  copyClipboardItem: "copy_clipboard_item",
  clipboardImage: "clipboard_image",
  pinClipboardItem: "pin_clipboard_item",
  clearClipboardHistory: "clear_clipboard_history",
} as const;
