pub const BAD_URLS: &[&str] = &[
    "http://push.example/p",
    "https://push.example:8443/p",
    "https://user@push.example/p",
    "https://127.0.0.1/p",
    "https://[2606:4700::1111]/p",
    "https://push.example/p#frag",
    "https://push.example/p\\q",
];

pub const BAD_IPS: &[&str] = &[
    "127.0.0.1",
    "10.0.0.1",
    "172.16.0.1",
    "192.168.0.1",
    "169.254.169.254",
    "100.64.0.1",
    "192.0.2.1",
    "224.0.0.1",
    "0.0.0.0",
    "240.0.0.1",
    "198.18.0.1",
    "203.0.113.1",
    "::1",
    "fd00::1",
    "fe80::1",
    "::ffff:127.0.0.1",
    "2001:db8::1",
];
