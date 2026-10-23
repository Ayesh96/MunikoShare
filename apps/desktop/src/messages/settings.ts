export const TAB_SETTINGS_LABEL = "Settings";

export const SETTINGS_DEVICE_TITLE = "This device";
export const SETTINGS_DEVICE_NAME_LABEL = "Device name";
export const SETTINGS_DEVICE_NAME_HINT = "Leave empty to use this computer's name. Applies after you restart Muniko Share.";
export const SETTINGS_SAVE_LABEL = "Save";
export const SETTINGS_NAME_SAVED = "Device name saved. Restart Muniko Share to apply it.";

export const SETTINGS_DOWNLOADS_TITLE = "Downloads";
export const SETTINGS_DOWNLOADS_HINT = "Received files are saved here. A new folder applies after you restart Muniko Share.";
export const SETTINGS_CHOOSE_FOLDER_LABEL = "Choose folder";
export const SETTINGS_RESET_FOLDER_LABEL = "Use default";
export const SETTINGS_OPEN_FOLDER_LABEL = "Open download folder";
export const SETTINGS_FOLDER_SAVED = "Download folder saved. Restart Muniko Share to apply it.";
export const SETTINGS_PICK_FOLDER_TITLE = "Choose the download folder";

export const SETTINGS_STARTUP_TITLE = "Startup";
export const SETTINGS_AUTOSTART_LABEL = "Start Muniko Share when I log in";

export const SETTINGS_PHONE_TITLE = "Phones";

export const SETTINGS_ABOUT_TITLE = "About";
export const settingsVersion = (version: string) => `Version ${version}`;
export const SETTINGS_CHECK_UPDATES_LABEL = "Check for updates";
export const SETTINGS_INSTALL_UPDATE_LABEL = "Install";
export const UPDATE_CHECKING = "Checking for updates…";
export const UPDATE_UP_TO_DATE = "You are up to date.";
export const UPDATE_INSTALLING = "Downloading the update…";
export const updateAvailable = (version: string) => `Update available: version ${version}.`;
export const updateError = (detail: string) => `Could not check for updates (${detail}).`;

export const SEND_TO_TITLE = "Send to…";
export const SEND_TO_SEND_LABEL = "Send";
export const SEND_TO_CANCEL_LABEL = "Cancel";
export const SEND_TO_PHONES_HEADING = "Phones";
export const SEND_TO_DEVICES_HEADING = "Paired devices";
export const SEND_TO_EMPTY_TITLE = "No paired devices or phones nearby";
export const SEND_TO_EMPTY_HINT = "Pair a device or connect a phone, then send again.";
export const sendToSummary = (count: number) => `${count} ${count === 1 ? "item" : "items"} to send`;
export const sendToStarted = (name: string) => `Sending to ${name}`;

export const PHONE_ACCESS_OFF_TITLE = "Phone access is off";
export const PHONE_ACCESS_OFF_HINT = "Turn it on in Settings to connect phones.";
