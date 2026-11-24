//! Roots overlap regressions: both containment directions are rejected.

#[test]
fn roots_reject_both_containment_directions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let requests = dir.path().join("requests");
    let disk = dir.path().join("disk");
    let nginx_inside_disk = disk.join("nginx");
    for path in [&requests, &disk, &nginx_inside_disk] {
        std::fs::create_dir(path).expect("dir");
    }
    let err = skrynia_publish_provider::roots::open(&requests, &disk, &nginx_inside_disk)
        .err()
        .expect("nginx inside disk must be rejected");
    assert!(err.contains("must not overlap"), "{err}");

    let dir2 = tempfile::tempdir().expect("tempdir2");
    let nginx = dir2.path().join("nginx");
    let disk_inside_nginx = nginx.join("disk");
    let requests2 = dir2.path().join("requests");
    for path in [&nginx, &disk_inside_nginx, &requests2] {
        std::fs::create_dir(path).expect("dir");
    }
    let err = skrynia_publish_provider::roots::open(&requests2, &disk_inside_nginx, &nginx)
        .err()
        .expect("disk inside nginx must be rejected");
    assert!(err.contains("must not overlap"), "{err}");
}
