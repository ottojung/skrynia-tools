mod common;

use common::{BAD_IPS, BAD_URLS, FakeResolver, one, resolver};
use std::collections::HashMap;
use webpush_sender::{Error, sub::validate_url};

#[test]
fn public_https_is_pinned() {
    let target = validate_url("https://push.example:443/push?a=1", &one("93.184.216.34"))
        .expect("public endpoint");
    assert_eq!(target.host, "push.example");
    assert_eq!(target.ip.to_string(), "93.184.216.34");
    assert_eq!(target.port, 443);
    assert_eq!(target.path, "/push?a=1");
}

#[test]
fn scheme_port_userinfo_and_ip_literals_fail() {
    let r = one("93.184.216.34");
    for url in BAD_URLS {
        assert!(
            matches!(validate_url(url, &r), Err(Error::Ssrf(_))),
            "{url}"
        );
    }
}

#[test]
fn private_and_special_answers_fail() {
    for ip in BAD_IPS {
        assert!(
            validate_url("https://push.example/p", &one(ip)).is_err(),
            "{ip}"
        );
    }
}

#[test]
fn mixed_or_missing_dns_fails() {
    let mut r = resolver(&[("push.example", "93.184.216.34")]);
    r.answers
        .get_mut("push.example")
        .unwrap()
        .push("127.0.0.1".parse().unwrap());
    assert!(validate_url("https://push.example/p", &r).is_err());
    let empty = FakeResolver {
        answers: HashMap::new(),
    };
    // A missing resolver answer is an injected I/O failure, not an SSRF
    // verdict; unsafe answers below remain Ssrf.
    assert_eq!(
        validate_url("https://push.example/p", &empty),
        Err(Error::Transport("dns-io"))
    );
}
