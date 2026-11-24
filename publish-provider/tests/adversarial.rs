//! Fail-closed behavior on hostile paths, sources and publication nodes.

mod support;

use support::{Fixture, call, request, source, target};

fn code(f: &Fixture, name: &str, ticket: u32, path: &[u8]) -> u32 {
    request(f, ticket, path);
    call(f, name, ticket)
}

#[test]
fn rejects_bad_tickets_and_paths() {
    let f = Fixture::start();
    assert_eq!(call(&f, "publish", 7), 1);
    let long = vec![b'a'; 4097];
    for path in [
        &b"../escape"[..],
        &b"/abs"[..],
        &b"a//b"[..],
        &b"a/"[..],
        &b"x\0y"[..],
        &b""[..],
        &long,
    ] {
        assert_eq!(code(&f, "publish", 8, path), 2);
    }
}

#[test]
fn rejects_conflicts_without_touching_real_files() {
    let f = Fixture::start();
    std::fs::create_dir_all(f.disk.join("apps")).expect("apps dir");
    std::os::unix::fs::symlink("elsewhere", f.disk.join("apps/link")).expect("source symlink");
    assert_eq!(code(&f, "publish", 10, b"apps/link"), 3);

    source(&f, "apps/real.txt", b"data");
    std::fs::create_dir_all(f.nginx.join("apps")).expect("destination dir");
    std::fs::write(f.nginx.join("apps/real.txt"), b"keep").expect("real file");
    assert_eq!(code(&f, "publish", 11, b"apps/real.txt"), 4);
    assert_eq!(
        std::fs::read(f.nginx.join("apps/real.txt")).unwrap(),
        b"keep"
    );

    source(&f, "apps/wrong.txt", b"data");
    assert_eq!(code(&f, "publish", 12, b"apps/wrong.txt"), 0);
    let wrong = f.nginx.join("apps/wrong.txt");
    std::fs::remove_file(&wrong).expect("remove link");
    std::os::unix::fs::symlink("/wrong", &wrong).expect("wrong link");
    assert_eq!(code(&f, "publish", 12, b"apps/wrong.txt"), 4);
    assert_eq!(code(&f, "unpublish", 12, b"apps/wrong.txt"), 4);

    source(&f, "apps/alias.txt", b"x");
    std::os::unix::fs::symlink(f.disk.join("apps"), f.nginx.join("apps-link")).unwrap();
    assert_eq!(code(&f, "publish", 13, b"apps/alias.txt"), 0);
    std::fs::create_dir_all(f.disk.join("apps/sub-link")).expect("sub-link");
    source(&f, "apps/sub-link/f.txt", b"x");
    std::os::unix::fs::symlink(f.disk.join("apps/sub"), f.nginx.join("apps/sub-link")).unwrap();
    assert_eq!(code(&f, "publish", 14, b"apps/sub-link/f.txt"), 4);
    assert!(target(&f, "apps/wrong.txt").is_some());
}
