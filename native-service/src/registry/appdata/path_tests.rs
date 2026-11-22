use super::path::{data_path, valid_key};

#[test]
fn arbitrary_valid_key_bytes_have_bounded_collision_resistant_component() {
    let namespace = [5u8; 32];
    for length in [1, 125, 126, 127, 255] {
        let key = vec![b'A'; length];
        assert!(valid_key(&key));
        let path = data_path("programs", &namespace, &key).unwrap();
        let file = path.rsplit('/').next().unwrap();
        assert_eq!(file.len(), 68);
        assert!(file.ends_with(".bin"));
        assert!(file[..64].bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(path, data_path("programs", &namespace, &key).unwrap());
    }
    assert!(!valid_key(&[]));
    assert!(!valid_key(&[0u8; 256]));
    assert!(data_path("programs", &namespace, &[]).is_none());
    assert!(data_path("programs", &namespace, &[0u8; 256]).is_none());
}

#[test]
fn key_changes_and_namespace_changes_cannot_alias() {
    let a = [7u8; 32];
    let b = [8u8; 32];
    let key = [b'A'; 255];
    let first = data_path("programs", &a, &key).unwrap();
    assert_ne!(first, data_path("programs", &b, &key).unwrap());
    let mut changed = key;
    changed[254] = b'B';
    assert_ne!(first, data_path("programs", &a, &changed).unwrap());
    assert_ne!(first, data_path("programs", &a, &[b'A'; 125]).unwrap());
}
