//! Typed failures of the board domain; transports map them to statuses by variant, never by text.

use axum::http::StatusCode;

use crate::messages::board as msg;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BoardError {
    #[error("{}", msg::NO_BOARD)]
    NoBoard,
    #[error("{}", msg::ALREADY_ON_BOARD)]
    AlreadyOnBoard,
    #[error("{}", msg::NOT_A_MEMBER)]
    NotMember,
    #[error("{}", msg::BOARD_MISMATCH)]
    BoardMismatch,
    #[error("{}", msg::CODE_REQUIRED)]
    CodeRequired,
    #[error("{}", msg::CODE_TOO_SHORT)]
    CodeTooShort,
    #[error("{}", msg::INVALID_NAME)]
    InvalidName,
    #[error("{}", msg::TEXT_EMPTY)]
    TextEmpty,
    #[error("{}", msg::TEXT_TOO_LARGE)]
    TextTooLarge,
    #[error("{}", msg::FILE_NAME_INVALID)]
    InvalidFileName,
    #[error("{}", msg::ITEM_NOT_FOUND)]
    ItemNotFound,
    #[error("{}", msg::NOT_YOUR_ITEM)]
    NotYourItem,
    #[error("{}", msg::ITEM_REJECTED)]
    ItemRejected,
    #[error("{}", msg::NOT_A_FILE)]
    NotAFile,
    #[error("{}", msg::NOT_A_FILE_ITEM)]
    NotAFileItem,
    #[error("{}", msg::BAD_FILE_ID)]
    BadFileId,
    #[error("{}", msg::NO_HOLDER)]
    NoHolder,
    #[error("{}", msg::FETCH_FAILED)]
    FetchFailed,
    #[error("{}", msg::FETCH_INTERRUPTED)]
    FetchInterrupted,
    #[error("{}", msg::HASH_MISMATCH)]
    HashMismatch,
    #[error("{}", msg::SIZE_MISMATCH)]
    SizeMismatch,
    /// A local storage failure (cache, export, read) with its cause.
    #[error("{0}")]
    Storage(String),
}

impl BoardError {
    pub fn storage(context: &str, detail: impl std::fmt::Display) -> Self {
        BoardError::Storage(format!("{context}: {detail}"))
    }

    /// The default HTTP status for a peer-facing board route.
    pub fn status(&self) -> StatusCode {
        match self {
            BoardError::NoBoard | BoardError::ItemNotFound | BoardError::NotAFileItem => StatusCode::NOT_FOUND,
            BoardError::NotMember | BoardError::NotYourItem => StatusCode::FORBIDDEN,
            BoardError::BoardMismatch | BoardError::AlreadyOnBoard => StatusCode::CONFLICT,
            BoardError::CodeRequired
            | BoardError::CodeTooShort
            | BoardError::InvalidName
            | BoardError::TextEmpty
            | BoardError::InvalidFileName
            | BoardError::ItemRejected
            | BoardError::NotAFile
            | BoardError::BadFileId => StatusCode::BAD_REQUEST,
            BoardError::TextTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            BoardError::NoHolder | BoardError::FetchFailed | BoardError::FetchInterrupted => StatusCode::SERVICE_UNAVAILABLE,
            BoardError::HashMismatch | BoardError::SizeMismatch | BoardError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MunikoError;

    #[test]
    fn display_comes_from_the_message_catalogue() {
        assert_eq!(BoardError::NoBoard.to_string(), msg::NO_BOARD);
        assert_eq!(BoardError::storage("ctx", "boom").to_string(), "ctx: boom");
    }

    #[test]
    fn variants_wrap_into_the_crate_error_and_keep_their_identity() {
        let error = MunikoError::from(BoardError::ItemNotFound);
        assert!(matches!(error, MunikoError::Board(BoardError::ItemNotFound)));
        assert!(error.to_string().ends_with(msg::ITEM_NOT_FOUND));
    }

    #[test]
    fn peer_route_statuses_follow_the_variant() {
        assert_eq!(BoardError::NoBoard.status(), StatusCode::NOT_FOUND);
        assert_eq!(BoardError::NotMember.status(), StatusCode::FORBIDDEN);
        assert_eq!(BoardError::BoardMismatch.status(), StatusCode::CONFLICT);
        assert_eq!(BoardError::TextTooLarge.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(BoardError::NoHolder.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(BoardError::HashMismatch.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
