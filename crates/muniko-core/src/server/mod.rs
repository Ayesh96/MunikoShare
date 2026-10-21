//! The shared HTTPS server: mutual TLS, local-network filter and pluggable routes.

mod acceptor;
mod http_server;
mod ip_filter;
mod middleware;
mod request_peer;
mod route_provider;
mod tls;
mod verifier;

#[cfg(test)]
mod tests;

pub use http_server::HttpServer;
pub use request_peer::{AuthenticatedPeer, PeerRejection, RequestPeer};
pub use route_provider::RouteProvider;
