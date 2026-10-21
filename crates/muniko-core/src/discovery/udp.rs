use std::net::{Ipv4Addr, SocketAddr};

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;

use crate::error::{MunikoError, Result};
use crate::messages::discovery as messages;

fn bind_error(error: std::io::Error) -> MunikoError {
    MunikoError::Discovery(format!("{}: {error}", messages::BROADCAST_BIND_FAILED))
}

/// Broadcast-capable socket that several instances on one machine can share the port of.
pub fn bind_broadcast_socket(port: u16) -> Result<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).map_err(bind_error)?;
    socket.set_reuse_address(true).map_err(bind_error)?;
    // Windows has no SO_REUSEPORT; SO_REUSEADDR already allows sharing there.
    #[cfg(unix)]
    socket.set_reuse_port(true).map_err(bind_error)?;
    socket.set_broadcast(true).map_err(bind_error)?;
    socket.set_nonblocking(true).map_err(bind_error)?;
    socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)).into()).map_err(bind_error)?;
    UdpSocket::from_std(socket.into()).map_err(bind_error)
}
