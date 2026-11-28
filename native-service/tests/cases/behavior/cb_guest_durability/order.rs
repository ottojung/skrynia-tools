use super::support::{call, dirs, disk_atom, opened_dirs, ops, path, sync_fds, wire};

fn assert_order(root: &str) {
    call::configure(None);
    disk_atom::write_atomic(&path(root), b"payload").expect("first write must succeed");
    let log = call::take_log();
    assert_eq!(
        ops(&log),
        super::expected::ops(),
        "exact operation ordering"
    );
    assert_eq!(
        opened_dirs(&log),
        dirs(root),
        "root plus every ancestor directory"
    );
    assert_eq!(
        sync_fds(&log),
        vec![100, 101, 102, 1, 1, 103],
        "ancestor dirs, file twice, then leaf dir"
    );
    let rename_at = ops(&log).iter().position(|&op| op == wire::RENAME).unwrap();
    assert_eq!(
        sync_fds(&log[..rename_at]),
        vec![100, 101, 102, 1, 1],
        "all ancestor dirs synced before rename"
    );
}

#[test]
fn user_first_write_syncs_ancestors_before_rename() {
    assert_order("user");
}

#[test]
fn operator_first_write_syncs_ancestors_before_rename() {
    assert_order("operator");
}
