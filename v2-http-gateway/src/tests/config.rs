use super::super::config;
#[test]
fn rejects_public_bind_and_remote_or_credentialed_upstreams() {
    let loopback = "127.0.0.1:31111".parse().unwrap();
    let wildcard = "0.0.0.0:31111".parse().unwrap();
    let good = "http://127.0.0.1:32222/".parse().unwrap();
    assert!(config::check(loopback, &good).is_ok());
    assert!(config::check(wildcard, &good).is_err());
    for url in [
        "https://127.0.0.1:32222/",
        "http://127.0.0.2:32222/",
        "http://evil.example:32222/",
        "http://127.0.0.1:32222/other",
        "http://127.0.0.1:32222/?token=secret",
        "http://admin:password@127.0.0.1:32222/",
        "http://127.0.0.1:32222/#frag",
        "http://127.0.0.1/",
    ] {
        assert!(
            config::check(loopback, &url.parse().unwrap()).is_err(),
            "{url} must not carry privileged kernel credentials"
        );
    }
}
