use std::net::IpAddr;

/// Non-loopback IPv4 addresses of this machine, as text for the UI.
pub fn lan_ipv4_addresses() -> Vec<String> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .map(|interface| interface.ip())
        .filter(|ip| matches!(ip, IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_link_local()))
        .map(|ip| ip.to_string())
        .collect()
}
