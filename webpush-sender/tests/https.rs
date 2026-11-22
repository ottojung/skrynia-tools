//! Executable pinned-IP TLS transport proofs against a disposable loopback CA.
mod common;

use common::tls::cases;

#[test]
fn host_sni_cert_and_required_headers() {
    cases::happy();
}

#[test]
fn pinned_ip_defeats_bad_dns_and_proxy_env() {
    cases::pinned();
}

#[test]
fn wrong_cert_and_wrong_host_rejected() {
    cases::wrong_trust();
}

#[test]
fn redirects_not_followed_and_statuses_classified() {
    cases::redirects_and_status();
}

#[test]
fn retry_after_timeout_and_body_bounds() {
    cases::limits();
}

#[test]
fn production_fails_closed_without_injected_trust() {
    cases::production();
}
