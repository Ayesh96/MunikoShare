export const TAB_PHONE_LABEL = "Phone";
export const QR_ALT = "QR code to connect a phone";

export const PHONE_NAME = "Phone";
export const phoneName = (label: string) => `Phone (${label})`;

export const PHONE_TITLE = "Phone companion";
export const PHONE_ACCESS_LABEL = "Phone access";
export const PHONE_ACCESS_HINT = "Let phones on this network send and receive files in a browser.";
export const PHONE_CONNECT_LABEL = "Connect phone";
export const PHONE_LIST_TITLE = "Connected phones";
export const PHONE_EMPTY_TITLE = "No phones connected";
export const PHONE_EMPTY_HINT = "Use Connect phone and scan the QR code with your phone camera.";
export const PHONE_SEND_FILES_LABEL = "Send files";
export const PHONE_SEND_TEXT_LABEL = "Send text";
export const PHONE_DISCONNECT_LABEL = "Disconnect";
export const PHONE_PICK_FILES_TITLE = "Choose files to send to the phone";
export const phoneRow = (connected: string, seen: string) => `Connected ${connected} · last seen ${seen}`;
export const phoneFilesSent = (count: number) => `${count} ${count === 1 ? "file" : "files"} ready on the phone`;
export const PHONE_TEXT_SENT = "Text shared with the phone";

export const PHONE_CONNECT_TITLE = "Connect a phone";
export const PHONE_CONNECT_STEPS = "Scan this code with the phone camera, or open the link below in its browser.";
export const PHONE_CERT_NOTE = "The phone will show a one-time certificate warning. Accept it to continue.";
export const PHONE_OTHER_LINKS = "Other network addresses:";
export const phoneExpiresIn = (time: string) => `Expires in ${time}`;
export const PHONE_EXPIRED = "This code has expired.";
export const PHONE_NEW_CODE_LABEL = "New code";
export const PHONE_CLOSE_LABEL = "Done";

export const PHONE_TEXT_TITLE = "Send text to phone";
export const PHONE_TEXT_PLACEHOLDER = "Text to show on the phone";
export const PHONE_TEXT_CONFIRM = "Send";
export const PHONE_CANCEL_LABEL = "Cancel";
