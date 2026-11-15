use super::{Arguments, options};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{collections::BTreeMap, fs};

#[test]
fn uses_explicit_pinned_ssh_connection() {
    let root =
        std::env::temp_dir().join(format!("skrynia-ssh-flags-{:032x}", rand::random::<u128>()));
    fs::create_dir(&root).unwrap();
    let key = root.join("key");
    let hosts = root.join("hosts");
    fs::write(&key, b"private key fixture").unwrap();
    fs::write(&hosts, b"example.org ssh-ed25519 fixture").unwrap();
    #[cfg(unix)]
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
    let mut flags = BTreeMap::new();
    for (name, value) in [
        ("ssh-host", "example.org"),
        ("ssh-user", "deployer"),
        ("ssh-port", "2200"),
        ("ssh-key", key.to_str().unwrap()),
        ("ssh-known-hosts", hosts.to_str().unwrap()),
    ] {
        flags.insert(format!("--{name}"), value.to_owned());
    }
    let args = Arguments::from_rpc("versions".into(), flags);
    let cmd = options::command(&args).unwrap();
    let text = cmd
        .get_args()
        .map(|v| v.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    for required in [
        "-F /dev/null",
        "-p 2200",
        "StrictHostKeyChecking=yes",
        "IdentityAgent=none",
        "GlobalKnownHostsFile=/dev/null",
        "ProxyCommand=none",
    ] {
        assert!(text.contains(required), "missing {required}");
    }
    fs::remove_dir_all(root).unwrap();
}
