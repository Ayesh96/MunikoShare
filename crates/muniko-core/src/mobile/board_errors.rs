//! Maps board failures to the HTTP answers the phone expects.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::board::BoardError;
use crate::error::MunikoError;
use crate::messages::mobile as msg;

pub fn board_error(error: &MunikoError) -> Response {
    let MunikoError::Board(error) = error else { return failed() };
    match error {
        BoardError::NoBoard => (StatusCode::CONFLICT, msg::NO_BOARD).into_response(),
        BoardError::TextEmpty | BoardError::InvalidFileName | BoardError::NotAFile => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
        BoardError::TextTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, error.to_string()).into_response(),
        BoardError::ItemNotFound | BoardError::NotAFileItem => (StatusCode::NOT_FOUND, msg::NOT_FOUND).into_response(),
        BoardError::NoHolder | BoardError::FetchFailed => (StatusCode::SERVICE_UNAVAILABLE, msg::BOARD_FILE_UNAVAILABLE).into_response(),
        _ => failed(),
    }
}

fn failed() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, msg::BOARD_FAILED).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_of(error: BoardError) -> StatusCode {
        board_error(&MunikoError::from(error)).status()
    }

    #[test]
    fn statuses_are_chosen_by_variant() {
        assert_eq!(status_of(BoardError::NoBoard), StatusCode::CONFLICT);
        assert_eq!(status_of(BoardError::TextEmpty), StatusCode::BAD_REQUEST);
        assert_eq!(status_of(BoardError::NotAFile), StatusCode::BAD_REQUEST);
        assert_eq!(status_of(BoardError::TextTooLarge), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(status_of(BoardError::ItemNotFound), StatusCode::NOT_FOUND);
        assert_eq!(status_of(BoardError::NotAFileItem), StatusCode::NOT_FOUND);
        assert_eq!(status_of(BoardError::NoHolder), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(status_of(BoardError::FetchFailed), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(status_of(BoardError::HashMismatch), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn a_storage_error_with_look_alike_text_stays_internal() {
        let lookalike = BoardError::Storage(crate::messages::board::NO_BOARD.to_string());
        assert_eq!(status_of(lookalike), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn non_board_errors_are_internal() {
        assert_eq!(board_error(&MunikoError::InvalidState("x".into())).status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
