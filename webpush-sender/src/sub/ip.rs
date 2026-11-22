use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Only global unicast addresses may be connected to. Rejects loopback,
/// RFC1918, link-local (incl. 169.254.169.254), CGNAT, multicast, reserved,
/// documentation, benchmarking, IPv6 ULA/link-local and IPv4-mapped v6.
pub fn is_global(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => !v4_blocked(v),
        IpAddr::V6(v) => !v6_blocked(v),
    }
}

fn v4_blocked(v: Ipv4Addr) -> bool {
    let [a, b, c, _] = v.octets();
    a == 0 // 0.0.0.0/8 ("this network") — not just the exact unspecified address
        || v.is_unspecified()
        || v.is_loopback()
        || v.is_private()
        || v.is_link_local()
        || v.is_multicast()
        || a >= 240
        || (a == 100 && (64..128).contains(&b))
        || (a == 192 && b == 0 && c == 2)
        || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
        || (a == 203 && b == 0 && c == 113)
}

fn v6_blocked(v: Ipv6Addr) -> bool {
    let s = v.segments();
    v.is_unspecified()
        || v.is_loopback()
        || v.is_multicast()
        || v.is_unicast_link_local()
        || v.is_unique_local()
        || (s[0] == 0x2001 && s[1] == 0xdb8) // documentation
        || (s[0] == 0x0064 && s[1] == 0xff9b) // 64:ff9b::/96 NAT64 (to_ipv4() is None)
        || s[0] == 0x2002 // 2002::/16 6to4
        || (s[0] == 0x2001 && s[1] == 0x0000) // 2001::/32 Teredo
        || v.to_ipv4().is_some()
}
