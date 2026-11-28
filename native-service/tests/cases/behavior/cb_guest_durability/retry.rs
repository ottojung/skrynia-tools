use super::support::{call, dirs, disk_atom, opened_dirs, ops, path, sync_fds};

fn assert_retry(root: &str) {
    let ds = dirs(root);
    for (fail_sync, synced_dir) in [
        (1usize, ds[0].clone()),
        (2, ds[1].clone()),
        (3, ds[2].clone()),
    ] {
        call::configure(Some(fail_sync));
        let failed = disk_atom::write_atomic(&path(root), b"payload");
        assert_eq!(failed, Err(-5), "sync of {synced_dir} must fail the write");
        let failed_log = call::take_log();
        assert!(
            !ops(&failed_log).contains(&super::support::wire::RENAME),
            "no rename may publish after failed ancestor sync of {synced_dir}"
        );

        call::configure(None);
        disk_atom::write_atomic(&path(root), b"payload").expect("retry must succeed");
        let retry_log = call::take_log();
        assert_eq!(
            opened_dirs(&retry_log),
            ds,
            "retry must re-sync all ancestors"
        );
        assert_eq!(
            sync_fds(&retry_log),
            vec![100, 101, 102, 1, 1, 103],
            "mkdir success is not durable existence"
        );
    }
}

#[test]
fn user_retry_after_each_ancestor_sync_failure() {
    assert_retry("user");
}

#[test]
fn operator_retry_after_each_ancestor_sync_failure() {
    assert_retry("operator");
}
