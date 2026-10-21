//! Network Board: a shared space every member device replicates and syncs by gossip.

mod bootstrap;
mod canonical;
mod client;
mod error;
mod config;
mod context;
mod files;
mod guard;
mod hex_serde;
mod join_code;
mod limits;
mod members;
mod membership;
mod memory;
mod model;
mod observed;
mod outbox;
mod probe;
mod replica;
mod routes;
mod runtime;
mod service;
mod signing;
mod sqlite;
mod store;
mod syncer;
mod view;
mod wire;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_files;

pub use config::BoardConfig;
pub use error::BoardError;
pub use files::FileLease;
pub use memory::MemoryBoardStore;
pub use model::{Board, BoardInfo, BoardItem, BoardMode, DigestEntry, ItemKind};
pub use observed::ObservedBoard;
pub use routes::BoardRoutes;
pub use runtime::BoardRuntime;
pub use service::{BoardService, OpenedFile};
pub use sqlite::SqliteBoardStore;
pub use store::BoardStore;
pub use view::{BoardItemView, ViewKind};
