export const TAB_BOARD_LABEL = "Board";

export const BOARD_MODE_OPEN = "Open";
export const BOARD_MODE_PROTECTED = "Protected";
export const BOARD_SAVED_BADGE = "Saved on this device";
export const BOARD_AUTHOR_OFFLINE = "Author offline";
export const BOARD_YOU = "You";

export const BOARD_COMPOSER_PLACEHOLDER = "Write something for everyone on the network";
export const BOARD_POST_LABEL = "Post";
export const BOARD_ADD_FILES_LABEL = "Add files";
export const BOARD_PICK_FILES_TITLE = "Choose files to post";
export const BOARD_FEED_TITLE = "Shared items";
export const BOARD_FEED_EMPTY_TITLE = "Nothing shared yet";
export const BOARD_FEED_EMPTY_HINT = "Posts and files appear here for every device on the network.";
export const BOARD_COPY_LABEL = "Copy";
export const BOARD_DOWNLOAD_LABEL = "Download";
export const BOARD_DELETE_LABEL = "Delete";
export const boardItemRow = (author: string, time: string) => `${author} · ${time}`;
export const boardFileRow = (size: string, author: string, time: string) => `${size} · ${author} · ${time}`;
export const boardDownloaded = (path: string) => `Saved to ${path}`;
export const BOARD_COPIED = "Copied to the clipboard";

export const BOARD_KEEPER_LABEL = "Keeper";
export const BOARD_KEEPER_HINT = "Keep a copy of every board file on this device.";
export const BOARD_LEAVE_LABEL = "Leave board";
export const BOARD_LEAVE_TITLE = "Leave board";
export const boardLeavePrompt = (name: string) => `Leave "${name}"? Its items stay on the other devices.`;

export const BOARD_SETUP_TITLE = "No board yet";
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
