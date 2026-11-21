mod support;

#[test]
fn negative_auth_host_origin_cross_app_and_no_leak() {
    support::negative::run();
}
