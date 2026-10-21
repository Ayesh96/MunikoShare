//! Muniko Share core: identity, discovery, transfer and node lifecycle.

pub mod board;
pub mod clipboard;
pub mod constants;
pub mod device;
pub mod discovery;
pub mod error;
pub mod events;
pub mod history;
pub mod identity;
pub mod info;
pub mod lifecycle;
pub mod messages;
pub mod mobile;
pub mod net;
pub mod node;
pub mod pairing;
pub mod server;
pub mod transfer;
pub mod trust;
pub mod util;
pub mod web;

pub use device::DeviceInfo;
pub use error::{MunikoError, Result};
pub use events::{EventBus, FileSummary, NodeEvent, TransferDirection};
pub use lifecycle::{Lifecycle, LifecycleState};
pub use node::{MunikoNode, NodeConfig};
