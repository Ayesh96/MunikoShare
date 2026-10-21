export const PAIRING_CHANGE_EVENTS: readonly string[] = ["pairingCompleted", "deviceUnpaired"];

export const CLIPBOARD_REFRESH_EVENTS: readonly string[] = ["clipboardReceived", "clipboardImageReceived", "clipboardSent"];
/** Local copies are captured by a watcher that emits no event, so the tab polls. */
export const CLIPBOARD_POLL_INTERVAL_MS = 3000;

export const BOARD_REFRESH_EVENTS: readonly string[] = ["boardChanged"];
/** A burst of board events (a sync) collapses into one reload. */
export const BOARD_REFRESH_DEBOUNCE_MS = 300;

export const PHONE_REFRESH_EVENTS: readonly string[] = ["phoneConnected", "phoneDisconnected"];
/** Phones disappear silently when their session expires, so the tab polls. */
export const PHONE_POLL_INTERVAL_MS = 10000;
export const COUNTDOWN_TICK_MS = 1000;
