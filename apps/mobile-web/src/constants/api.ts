export const API_BASE = "/api/m";
export const SESSION_PATH = `${API_BASE}/session`;
export const UPLOAD_PATH = `${API_BASE}/upload`;
export const INBOX_PATH = `${API_BASE}/inbox`;
export const TEXT_PATH = `${API_BASE}/text`;
export const BOARD_PATH = `${API_BASE}/board`;
export const BOARD_TEXT_PATH = `${BOARD_PATH}/text`;
export const BOARD_FILES_PATH = `${BOARD_PATH}/files`;
export const EVENTS_PATH = `${API_BASE}/events`;

export const FILE_NAME_HEADER = "x-muniko-file-name";
export const TOKEN_HASH_KEY = "t";
export const POLL_INTERVAL_MS = 5000;
export const SERVICE_WORKER_FILE = "sw.js";

export const EVENT_INBOX = "inbox";
export const EVENT_TEXT = "text";
export const EVENT_BOARD = "board";
export const EVENT_REVOKED = "revoked";

export const HTTP_UNAUTHORIZED = 401;
export const HTTP_NO_CONTENT = 204;
