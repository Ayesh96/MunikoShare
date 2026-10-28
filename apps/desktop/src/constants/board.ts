export const BOARD_COMMANDS = {
  state: "board_state",
  postText: "board_post_text",
  postFiles: "board_post_files",
  postImage: "board_post_image",
  copyImage: "board_copy_image",
  image: "board_image",
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

/** Mirrors the desktop shell's IMAGE_NAME_HEADER; the request body is the raw PNG. */
export const IMAGE_NAME_HEADER = "x-file-name";
export const PNG_MIME = "image/png";
/** Board files with this extension show in the shared clipboard as images. */
export const IMAGE_EXTENSION = ".png";
/** Larger images get no preview, so a big file is never fetched just to draw a thumbnail. */
export const IMAGE_PREVIEW_MAX_BYTES = 10 * 1024 * 1024;
