export const APP_NAME = "Muniko Share";
export const THIS_DEVICE_LABEL = "This device:";
export const ADD_BY_IP_LABEL = "Add device by IP";
export const NO_ADDRESS = "no network address";
export const deviceEndpoint = (addresses: string[], port: number) =>
  `${addresses.length > 0 ? addresses.join(", ") : NO_ADDRESS} · port ${port}`;
export const LOADING_DEVICE = "Starting...";

export const TAB_DEVICES_LABEL = "Devices";
export const TAB_HISTORY_LABEL = "History";

export const NEARBY_TITLE = "Nearby devices";
export const NEARBY_EMPTY_TITLE = "No devices found";
export const NEARBY_EMPTY_HINT = "Open Muniko Share on another device on the same Wi-Fi or network, or add one by its IP address.";
export const SEND_FILES_LABEL = "Send files";
export const SEND_CLIPBOARD_LABEL = "Send clipboard";
export const PAIR_LABEL = "Pair";
export const UNPAIR_LABEL = "Unpair";
export const PAIRED_BADGE = "Paired";
export const PICK_FILES_TITLE = "Choose files to send";
export const SEND_FOLDER_LABEL = "Send folder";
export const PICK_FOLDER_TITLE = "Choose a folder to send";
export const AUTO_ACCEPT_LABEL = "Auto-accept";

export const HISTORY_TITLE = "History";
export const HISTORY_EMPTY_TITLE = "No history yet";
export const HISTORY_EMPTY_HINT = "Sent and received files, pairing and clipboard show up here.";
export const CLEAR_HISTORY_LABEL = "Clear history";
export const HISTORY_EMPTY_ACTION = "Go to devices";
export const HISTORY_ERROR_TITLE = "History didn't load";
export const HISTORY_ERROR_HINT = "Muniko Share couldn't read the saved history file. Try again, or restart the app if this repeats.";
export const RETRY_LABEL = "Try again";

export const BADGE_RECEIVED = "Received";
export const BADGE_SENT = "Sent";
export const BADGE_FAILED = "Failed";
export const BADGE_PAIRED = "Paired";
export const BADGE_UNPAIRED = "Unpaired";
export const BADGE_CLIPBOARD_RECEIVED = "Clipboard in";
export const BADGE_CLIPBOARD_SENT = "Clipboard out";

export const historyFileReceived = (name: string) => `Received ${name}`;
export const historyFileSent = (name: string) => `Sent ${name}`;
export const HISTORY_CLIPBOARD_SENT = "Clipboard sent";
export const HISTORY_PAIRED = "Paired";
export const HISTORY_UNPAIRED = "Unpaired";
export const historyRow = (device: string, time: string) => `${device} · ${time}`;
export const clipboardSentText = (to: string) => `Clipboard sent to ${to}`;
export const clipboardReceivedBanner = (from: string, preview: string) =>
  `Clipboard received from ${from}: ${preview}`;

export const PAIR_ENTER_TITLE = "Pair device";
export const pairEnterPrompt = (device: string) => `Enter the 6-digit code shown on ${device}`;
export const PAIR_CONFIRM_LABEL = "Confirm";
export const PAIR_CANCEL_LABEL = "Cancel";
export const PAIR_SHOW_TITLE = "Pairing request";
export const pairShowPrompt = (device: string) => `${device} wants to pair. Enter this code on that device:`;

export const OFFER_TITLE = "Incoming files";
export const offerPrompt = (device: string) => `${device} wants to send you:`;
export const offerTotal = (size: string) => `Total: ${size}`;
export const ACCEPT_LABEL = "Accept";
export const DECLINE_LABEL = "Decline";

export const TRANSFERS_TITLE = "Transfers";
export const DIRECTION_INCOMING = "from";
export const DIRECTION_OUTGOING = "to";
export const transferLabel = (direction: string, device: string, percent: number, done: string, total: string) =>
  `${direction} ${device} · ${percent}% (${done} / ${total})`;

export const SEND_TEXT_TITLE = "Send text";
export const SEND_TEXT_PLACEHOLDER = "Type text to send to a paired device";
export const sendTextTo = (device: string) => `Send text to ${device}`;
export const SEND_TEXT_NO_PAIRED = "Pair a device to send text.";

export const CLOSE_LABEL = "Close";
export const DISMISS_LABEL = "Dismiss";

export const ADD_DEVICE_TITLE = "Add device by IP";
export const ADD_DEVICE_IP_LABEL = "IP address";
export const ADD_DEVICE_IP_PLACEHOLDER = "192.168.1.20";
export const ADD_DEVICE_PORT_LABEL = "Port";
export const ADD_DEVICE_CONFIRM_LABEL = "Add";
export const ADD_DEVICE_INVALID_PORT = "Enter a port between 1 and 65535";
export const deviceAdded = (name: string) => `Added ${name}`;

export const waitingForAccept = (device: string) => `Waiting for ${device} to accept…`;

export const TAB_CLIPBOARD_LABEL = "Clipboard";
export const CLIPBOARD_SYNC_LABEL = "Clipboard sync";
export const CLIPBOARD_SYNC_HINT =
  "Clipboard sync is automatic both ways, but only when it is turned on on both devices.";

export const CLIPBOARD_HISTORY_TITLE = "Clipboard history";
export const CLIPBOARD_EMPTY_TITLE = "Clipboard history is empty";
export const CLIPBOARD_EMPTY_HINT = "Copied and received items show up here.";
export const CLEAR_CLIPBOARD_LABEL = "Clear";
export const COPY_LABEL = "Copy";
export const PIN_LABEL = "Pin";
export const UNPIN_LABEL = "Unpin";
export const PINNED_LABEL = "Pinned";
export const SOURCE_THIS_DEVICE = "This device";
export const IMAGE_ALT = "Clipboard image";
export const clipboardTextLength = (length: number) => `${length} characters`;
export const clipboardImageTitle = (width: number, height: number) => `Image ${width}×${height}`;
export const clipboardItemRow = (detail: string, source: string, time: string) => `${detail} · ${source} · ${time}`;
export const clipboardImageBanner = (from: string, width: number, height: number) =>
  `Image (${width}×${height}) from ${from} copied to your clipboard`;

export const hotkeySent = (count: number) => `Clipboard sent to ${count} ${count === 1 ? "device" : "devices"}`;
export const HOTKEY_NO_DEVICES = "No paired devices nearby to send the clipboard to";
