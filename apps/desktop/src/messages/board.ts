import { PASTED_IMAGE_PREFIX } from "../constants/board";

export const TAB_BOARD_LABEL = "Files";

export const BOARD_MODE_OPEN = "Open";
export const BOARD_MODE_PROTECTED = "Protected";
export const BOARD_AUTHOR_OFFLINE = "Author offline";
export const BOARD_YOU = "You";

export const BOARD_ADD_FILES_LABEL = "Share files";
export const BOARD_SHARE_TITLE = "Share files with everyone";
export const BOARD_DROP_HINT = "Pick files, or drop them anywhere in this window. Every device on the network can download them.";
export const BOARD_PICK_FILES_TITLE = "Choose files to post";
export const BOARD_FILES_TITLE = "Shared files";
export const BOARD_FILES_EMPTY_TITLE = "No files shared yet";
export const BOARD_FILES_EMPTY_HINT = "Files anyone on the network shares show up here, ready to download.";
export const BOARD_COPY_LABEL = "Copy";
export const BOARD_DOWNLOAD_LABEL = "Download";
export const BOARD_DELETE_LABEL = "Delete";
export const boardItemRow = (author: string, time: string) => `${author} · ${time}`;
export const boardFileRow = (size: string, author: string, time: string) => `${size} · ${author} · ${time}`;
export const boardDownloaded = (path: string) => `Saved to ${path}`;
export const BOARD_COPIED = "Copied to the clipboard";
export const BOARD_IMAGE_COPIED = "Image copied to the clipboard";
export const BOARD_IMAGE_SHARED = "Image shared with everyone";
export const BOARD_IMAGE_ALT = "Shared image";
export const BOARD_IMAGE_TITLE = "Image";
export const pastedImageName = (stamp: string) => `${PASTED_IMAGE_PREFIX}${stamp}.png`;

export const BOARD_SETTINGS_TITLE = "Shared board";
export const BOARD_SETTINGS_HINT = "The board that File Sharing posts to. Devices on the same board see each other's posts and files.";
export const BOARD_KEEPER_LABEL = "Keep a copy of every shared file";
export const BOARD_KEEPER_HINT = "Saves each file on this device, so others can still download it when the person who shared it is offline.";
export const BOARD_LEAVE_LABEL = "Leave board";
export const BOARD_LEAVE_TITLE = "Leave board";
export const boardLeavePrompt = (name: string) => `Leave "${name}"? Its items stay on the other devices.`;

export const BOARD_SETUP_TITLE = "No board yet";
export const BOARD_PENDING_TITLE = "Getting sharing ready";
export const BOARD_PENDING_HINT = "This takes a few seconds. You can also pick or create a board in Settings.";
export const BOARD_OPEN_SETTINGS_LABEL = "Open settings";
export const BOARD_CONFLICT_NOTICE = "Another shared board was found on this network.";
export const BOARD_CONFLICT_REVIEW = "Review in Settings";
export const BOARD_SETUP_HINT = "A board is being set up automatically. You can also create one or join one found on the network.";
export const BOARD_CREATE_LABEL = "Create board";
export const BOARD_DISCOVERED_TITLE = "Boards on this network";
export const BOARD_JOIN_LABEL = "Join";
export const boardDiscoveredRow = (peer: string, members: number) => `Hosted by ${peer} · ${members} ${members === 1 ? "device" : "devices"}`;

export const BOARD_CREATE_TITLE = "Create board";
export const BOARD_NAME_LABEL = "Board name";
export const BOARD_NAME_PLACEHOLDER = "Office";
export const BOARD_PROTECTED_SWITCH = "Require a join code";
export const BOARD_CODE_LABEL = "Join code";
export const BOARD_CODE_HINT = "Share this code with the people who should join.";
export const BOARD_CREATE_CONFIRM = "Create";
export const BOARD_CANCEL = "Cancel";

export const boardJoinTitle = (name: string) => `Join "${name}"`;
export const BOARD_JOIN_CODE_PROMPT = "Enter the join code for this board.";
export const BOARD_JOIN_OPEN_PROMPT = "This board is open to everyone on the network.";
export const BOARD_JOIN_CONFIRM = "Join";

export const BOARD_CONFLICT_ACTION = "Join instead";
export const boardConflict = (name: string, peer: string) => `Another board, "${name}", exists on this network (from ${peer}).`;
export const BOARD_CONFIRM_LABEL = "Confirm";

export const BOARD_SHOW_MORE_LABEL = "Show more";
export const BOARD_SHOW_LESS_LABEL = "Show less";
export const BOARD_RETRY_JOIN_LABEL = "Try again";
export const boardJoinFailed = (name: string) =>
  `Could not join ${name}, and your previous board was already left. Try again, or create a new board.`;
