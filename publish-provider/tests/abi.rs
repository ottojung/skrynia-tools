//! Pure ABI and path-grammar checks, independent of ambient I/O.

use skrynia_publish_provider::abi;

#[test]
fn typed_i32_little_endian_codec_is_exact() {
    assert_eq!(abi::decode(&[1, 1, 0x32, 4, 3, 2, 1]), Ok(0x0102_0304));
    assert_eq!(
        abi::encode(abi::DEST_CONFLICT),
        vec![1, 1, 0x32, 4, 0, 0, 0]
    );
    for value in [0, 1, 0x8000_0000, u32::MAX] {
        assert_eq!(abi::decode(&abi::encode(value)), Ok(value));
    }
    for bad in [
        &[][..],
        &[0, 0, 0, 0],
        &[2, 1, 0x32, 4, 3, 2, 1],
        &[1, 0, 0x32, 4, 3, 2, 1],
        &[1, 1, 0x40, 4, 3, 2, 1, 0, 0, 0, 0],
        &[1, 1, 0x32, 4, 3, 2],
        &[1, 1, 0x32, 4, 3, 2, 1, 0],
    ] {
        assert_eq!(abi::decode(bad), Err(abi::BAD_TICKET));
    }
}

#[test]
fn syscalls_list_advertises_both_names() {
    assert_eq!(
        abi::syscalls_list(),
        serde_json::json!({"functions": [
            {"name": "publish", "params": ["i32"], "results": ["i32"]},
            {"name": "unpublish", "params": ["i32"], "results": ["i32"]}
        ]})
    );
}
