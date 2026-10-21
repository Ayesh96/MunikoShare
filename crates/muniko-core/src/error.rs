use crate::board::BoardError;
use crate::messages::errors;

pub type Result<T> = std::result::Result<T, MunikoError>;

#[derive(Debug, thiserror::Error)]
pub enum MunikoError {
    #[error("{}: {}", errors::IDENTITY_FAILED, .0)]
    Identity(String),
    #[error("{}: {}", errors::DISCOVERY_FAILED, .0)]
    Discovery(String),
    #[error("{}: {}", errors::TRANSFER_FAILED, .0)]
    Transfer(String),
    #[error("{}: {}", errors::SERVER_FAILED, .0)]
    Server(String),
    #[error("{}: {}", errors::NETWORK_FAILED, .0)]
    Network(String),
    #[error("{}: {}", errors::CLIPBOARD_FAILED, .0)]
    Clipboard(String),
    #[error("{}: {}", errors::BOARD_FAILED, .0)]
    Board(#[from] BoardError),
    #[error("{}: {}", errors::IO_FAILED, .0)]
    Io(#[from] std::io::Error),
    #[error("{}: {}", errors::PEER_NOT_FOUND, .0)]
    PeerNotFound(String),
    #[error("{}: {}", errors::INVALID_STATE, .0)]
    InvalidState(String),
}
