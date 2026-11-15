use super::Fixture;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, process::Command};
pub fn create() -> Fixture {
    let root = std::env::temp_dir().join(format!(
        "skrynia-deploy-test-{:032x}",
        rand::random::<u128>()
    ));
    let source = root.join("source");
    let apps = root.join("hosted");
    fs::create_dir_all(source.join("apps/site")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::write(source.join("apps/site/Makefile"),
        "build:\n\tmkdir -p build/assets\n\tcp ../../shared.txt build/index.html\n\tcp ../../shared.txt build/assets/data.txt\n").unwrap();
    let init = Command::new("git")
        .current_dir(&source)
        .args(["init", "-b", "main"])
        .output()
        .unwrap();
    assert!(init.status.success());
    fs::write(
        root.join("gitconfig"),
        format!(
            "[url \"file://{}\"]\n    insteadOf = https://github.com/owner/repo\n",
            source.display()
        ),
    )
    .unwrap();
    let fake = root.join("bin/podman");
    fs::write(&fake, "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo true; exit 0; fi\nwhile [ $# -gt 0 ]; do\n case \"$1\" in\n -v) src=\"${2%%:*}\"; shift 2;;\n -w) dir=\"$2\"; shift 2;;\n *) shift;;\n esac\ndone\ncd \"$src/${dir#/workspace/}\" || exit 1\nmake build\n").unwrap();
    #[cfg(unix)]
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    Fixture { root, source, apps }
}
