export const BOARD_COMMANDS = {
  state: "board_state",
  postText: "board_post_text",
  postFiles: "board_post_files",
  download: "board_download",
  delete: "board_delete",
  create: "board_create",
  join: "board_join",
  leave: "board_leave",
  setKeeper: "board_set_keeper",
} as const;

export const PHONE_COMMANDS = {
  invite: "phone_invite",
  sessions: "phone_sessions",
  revoke: "phone_revoke",
  sendFiles: "phone_send_files",
  shareText: "phone_share_text",
  setAccess: "phone_set_access",
  access: "phone_access",
} as const;

/** Mirrors muniko_core PHONE_ID_PREFIX: phone senders appear as "phone:<session id>". */
export const PHONE_DEVICE_PREFIX = "phone:";

export const QR_SIZE_PX = 220;
export const QR_MARGIN_MODULES = 1;

/** Text items longer than this collapse behind "Show more". */
export const BOARD_TEXT_COLLAPSE_LENGTH = 160;
