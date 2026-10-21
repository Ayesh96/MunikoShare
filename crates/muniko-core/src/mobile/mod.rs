//! Phone companion: token-gated web page and API for phones without the app.

mod attempts;
mod auth;
mod board_bridge;
mod board_errors;
mod board_routes;
mod cookie;
mod event;
mod events_route;
mod file_response;
mod inbox;
mod inbox_routes;
mod limits;
mod phone_ops;
mod range;
mod routes;
mod secret;
mod service;
mod session;
mod session_routes;
mod sessions;
mod spa;
mod sweeper;
mod text_routes;
mod tokens;
mod upload;
mod user_agent;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_board;
#[cfg(test)]
mod tests_limits;

pub use inbox::InboxView;
pub use board_bridge::MobileBoardBridge;
pub use limits::MobileLimits;
pub use routes::MobileRoutes;
pub use service::{Invite, MobileService};
pub use session::PhoneSession;
pub use sweeper::MobileSweeper;
