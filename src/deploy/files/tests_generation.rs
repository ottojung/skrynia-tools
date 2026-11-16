use super::{ensure_dirs, generation};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[test]
fn activation_clock_persists_and_refuses_invalid_state() {
    let root = std::env::temp_dir().join(format!("skrynia-clock-{:032x}", rand::random::<u128>()));
    ensure_dirs(&root).unwrap();
    assert_eq!(generation::current(&root, "myapp").unwrap(), 0);
    assert_eq!(generation::advance(&root, "myapp").unwrap(), 1);
    assert_eq!(generation::advance(&root, "myapp").unwrap(), 2);
    assert_eq!(generation::current(&root, "myapp").unwrap(), 2);
    let counter = root.join(".locks/myapp.generation");
    assert_eq!(fs::read(&counter).unwrap(), 2u64.to_be_bytes());
    fs::write(&counter, b"invalid").unwrap();
    assert!(generation::current(&root, "myapp").is_err());
    // Do not change the public app pointer when the durable clock is invalid.
    assert!(super::switch(&root, "myapp", "0123456789ab-0123456789abcdef").is_err());
    assert!(!root.join("apps/myapp").exists());
    #[cfg(unix)]
    {
        fs::remove_file(&counter).unwrap();
        symlink("/dev/null", &counter).unwrap();
        assert!(generation::current(&root, "myapp").is_err());
    }
    fs::remove_dir_all(root).unwrap();
}
