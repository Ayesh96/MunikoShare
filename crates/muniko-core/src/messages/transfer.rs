// Strings for the transfer module.
pub const FILE_RECEIVED: &str = "File received";
pub const REQUEST_REJECTED: &str = "File request rejected";

pub const MISSING_HEADER: &str = "Missing or invalid header";
pub const SIZE_MISMATCH: &str = "File size does not match the declared size";
pub const CHECKSUM_MISMATCH: &str = "File checksum does not match";
pub const UPLOAD_INTERRUPTED: &str = "Upload interrupted";
pub const STORE_FAILED: &str = "Could not store the file";

pub const READ_FILE_FAILED: &str = "Could not read the file";
pub const NOT_A_FILE: &str = "Path is not a file";
pub const SEND_FAILED: &str = "Could not send the file";
pub const PEER_REJECTED: &str = "Peer rejected the file";

pub const NOT_TRUSTED: &str = "Device is not trusted";
pub const DECLINED: &str = "declined";
pub const OFFER_TIMED_OUT: &str = "The offer was not answered in time";
pub const EMPTY_OFFER: &str = "Offer has no files";
pub const TOO_MANY_FILES: &str = "Offer has too many files";
pub const INVALID_CHECKSUM: &str = "Offer has an invalid checksum";
pub const UNKNOWN_TRANSFER: &str = "Unknown transfer";
pub const UNKNOWN_FILE: &str = "Unknown file index";
pub const WRONG_DEVICE: &str = "Transfer belongs to another device";
pub const FILE_ALREADY_SENT: &str = "File already sent or in progress";
pub const UNKNOWN_OFFER: &str = "Unknown or closed offer";
pub const OFFER_ACCEPTED_LOG: &str = "Offer accepted";

pub const NO_FILES_TO_SEND: &str = "No files to send";
pub const PEER_DECLINED: &str = "The other device declined the files";
pub const PEER_NOT_TRUSTING: &str = "The other device does not trust this device";
pub const PEER_TIMED_OUT: &str = "The other device did not answer in time";
pub const BAD_OFFER_RESPONSE: &str = "Unreadable response to the offer";

pub const INVALID_PATH: &str = "Invalid file path";
pub const OFFER_TOO_LARGE: &str = "Offer is too large";
pub const OFFSET_MISMATCH: &str = "Offset does not match the bytes already received";
pub const UPLOAD_INCOMPLETE: &str = "Upload ended before the whole file arrived";
pub const FILE_BUSY: &str = "File upload is in progress";
pub const OFFER_AUTO_ACCEPTED_LOG: &str = "Offer auto-accepted";
pub const BAD_RECEIVED_RESPONSE: &str = "Unreadable resume status from the peer";
pub const UPLOAD_RESUMING: &str = "Upload interrupted, resuming";
