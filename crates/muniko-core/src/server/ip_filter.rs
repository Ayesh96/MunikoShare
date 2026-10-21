//! Only devices on the local network may talk to the server.

use std::net::{IpAddr, Ipv6Addr};

pub fn is_local_network(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || is_unique_local(&v6) || is_link_local(&v6),
    }
}

// fc00::/7
fn is_unique_local(ip: &Ipv6Addr) -> bool {
    ip.segments()[0] & 0xfe00 == 0xfc00
}

// fe80::/10
fn is_link_local(ip: &Ipv6Addr) -> bool {
    ip.segments()[0] & 0xffc0 == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed(text: &str) -> bool {
        is_local_network(text.parse().unwrap())
    }

    #[test]
    fn allows_local_ranges() {
        for ip in ["10.1.2.3", "172.16.0.1", "172.31.255.1", "192.168.1.5", "127.0.0.1", "169.254.1.1", "::1", "fd00::1", "fe80::1", "::ffff:192.168.1.5"] {
            assert!(allowed(ip), "{ip}");
        }
    }

    #[test]
    fn blocks_public_ranges() {
        for ip in ["8.8.8.8", "172.32.0.1", "11.0.0.1", "2001:4860:4860::8888", "::ffff:8.8.8.8"] {
            assert!(!allowed(ip), "{ip}");
        }
    }
}
