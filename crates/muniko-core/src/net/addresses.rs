use std::net::IpAddr;

/// IPv4 first; scope-less IPv6 link-local addresses cannot connect, so they are skipped.
pub fn candidate_addresses(addresses: &[IpAddr]) -> Vec<IpAddr> {
    let (v4, v6): (Vec<IpAddr>, Vec<IpAddr>) = addresses.iter().copied().partition(IpAddr::is_ipv4);
    let usable_v6 = v6.into_iter().filter(|ip| !is_ipv6_link_local(ip));
    v4.into_iter().chain(usable_v6).collect()
}

fn is_ipv6_link_local(ip: &IpAddr) -> bool {
    matches!(ip, IpAddr::V6(v6) if v6.segments()[0] & 0xffc0 == 0xfe80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_ipv4_first_and_skips_link_local() {
        let input: Vec<IpAddr> = ["fe80::1", "fd00::2", "192.168.1.5", "10.0.0.2"].iter().map(|s| s.parse().unwrap()).collect();
        let expected: Vec<IpAddr> = ["192.168.1.5", "10.0.0.2", "fd00::2"].iter().map(|s| s.parse().unwrap()).collect();
        assert_eq!(candidate_addresses(&input), expected);
    }
}
