use serde_json::json;
use skrynia_tools::sdk::{
    DiscoveryError, ScalarError, ValType, cas1, fixed, names, parse_list, parse_version, read1,
    seek, status, write1,
};

#[test]
fn fixed_inventory_is_byte_only_and_u64_addressed() {
    assert_eq!(names().count(), 19);
    let read = fixed("read1").unwrap();
    assert_eq!(read.name, "read1");
    assert_eq!(read.params, [ValType::I32]);
    let write = fixed("write1").unwrap();
    assert_eq!(write.params, [ValType::I32, ValType::I32]);
    let seek_sig = fixed("seek").unwrap();
    assert_eq!(seek_sig.params, [ValType::I32, ValType::I64]);
    assert_eq!(seek_sig.results, [ValType::I64]);
    let cas = fixed("cas1").unwrap();
    assert_eq!(cas.params.len(), 4);
    assert_eq!(fixed("eval").unwrap().params.len(), 5);
    let root = fixed("open_directory").unwrap();
    assert_eq!(root.params, [ValType::I64, ValType::I64]);
    assert!(fixed("copy").is_none());
}

#[test]
fn scalar_encodings_follow_k0_choices() {
    assert_eq!(read1(255), Ok(255));
    assert_eq!(read1(-1), Err(ScalarError(-1)));
    assert!(read1(-1).unwrap_err().is_eof());
    assert_eq!(write1(0), Ok(()));
    assert_eq!(write1(-2), Err(ScalarError(-2)));
    assert_eq!(seek(12), Ok(12));
    assert_eq!(seek(-6), Err(ScalarError(-6)));
    assert_eq!(cas1(0), Ok(0));
    assert_eq!(status(0), Ok(()));
}

#[test]
fn discovery_accepts_list_version_and_empty_functions() {
    let list = json!({"functions":[{"name":"publish","params":["i32","i64"],"results":[]}]});
    let sigs = parse_list(&list).unwrap();
    assert_eq!(sigs[0].params, vec![ValType::I32, ValType::I64]);
    assert!(sigs[0].results.is_empty());
    let version = json!({"version":"1","commit":"abc","syscalls":[]});
    assert!(parse_version(&version).unwrap().is_empty());
    assert!(parse_list(&json!({"syscalls":[]})).is_err());
    let bad = json!({"functions":[{"name":"x","params":["f32"],"results":[]}]});
    assert_eq!(parse_list(&bad), Err(DiscoveryError::BadType));
    let dup = json!({"functions":[{"name":"x","params":[],"results":[]},{"name":"x","params":[],"results":[]}]});
    assert_eq!(parse_list(&dup), Err(DiscoveryError::DuplicateName));
}
