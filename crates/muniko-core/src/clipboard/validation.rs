use axum::http::StatusCode;

use crate::messages::clipboard as msg;

/// Why a piece of clipboard content may not be pushed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentProblem {
    EmptyText,
    TextTooLarge,
    EmptyImage,
    ImageTooLarge,
    InvalidImage,
}

impl ContentProblem {
    pub fn message(self) -> &'static str {
        match self {
            Self::EmptyText => msg::EMPTY_TEXT,
            Self::TextTooLarge => msg::TEXT_TOO_LARGE,
            Self::EmptyImage => msg::IMAGE_EMPTY,
            Self::ImageTooLarge => msg::IMAGE_TOO_LARGE,
            Self::InvalidImage => msg::IMAGE_INVALID,
        }
    }

    pub fn status(self) -> StatusCode {
        match self {
            Self::TextTooLarge | Self::ImageTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::EmptyText | Self::EmptyImage | Self::InvalidImage => StatusCode::BAD_REQUEST,
        }
    }
}
