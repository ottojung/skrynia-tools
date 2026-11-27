use serde_json::json;
use skrynia_tools::sdk::{DiscoveryError, MODULE, ValType, parse_list, parse_version, version_abi};

#[test]
fn discovery_accepts_list_and_exact_version_identity() {
    let list = json!({"functions":[{"name":"publish","params":["i32","i64"],"results":[]}]});
    let sigs = parse_list(&list).unwrap();
    assert_eq!(sigs[0].params, vec![ValType::I32, ValType::I64]);
    let version = json!({"version":"1","commit":"abc","abi":"skrynia_v3_1","syscalls":[]});
    assert_eq!(version_abi(&version), Ok(MODULE));
    assert!(parse_version(&version).unwrap().is_empty());
    assert_eq!(
        parse_version(&json!({"version":"1","commit":"abc","syscalls":[]})),
        Err(DiscoveryError::MissingAbi)
    );
    assert_eq!(
        version_abi(&json!({"abi":"skrynia"})),
        Err(DiscoveryError::BadAbi)
    );
    let bad = json!({"functions":[{"name":"x","params":["f32"],"results":[]}]});
    assert_eq!(parse_list(&bad), Err(DiscoveryError::BadType));
}
