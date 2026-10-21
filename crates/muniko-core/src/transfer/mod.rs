//! File transfer: offer/accept protocol, routes that receive files and a client that sends them.

mod client;
mod digest;
mod folder_roots;
mod incoming;
mod manifest;
mod partial;
mod peer_reply;
mod progress;
mod receiver;
mod rejection;
mod responder;
mod routes;
mod upload;
mod walk;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_folders_resume;

pub use client::TransferClient;
pub use responder::OfferResponder;
pub use routes::TransferRoutes;
