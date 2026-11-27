use skrynia_tools::sdk::{DeployCheck, ImportNeed, Signature, ValType, check_imports};

fn need(module: &str, name: &str, params: &[ValType], results: &[ValType]) -> ImportNeed {
    ImportNeed {
        module: module.into(),
        name: name.into(),
        params: params.to_vec(),
        results: results.to_vec(),
    }
}

#[test]
fn deployment_rejects_missing_mismatched_and_duplicate_ffi_imports() {
    let avail = [Signature::new("publish", &[ValType::I32], &[])];
    let publish = need("syscall", "publish", &[ValType::I32], &[]);
    assert_eq!(
        check_imports(std::slice::from_ref(&publish), &avail),
        DeployCheck::Ready
    );
    let missing = need("syscall", "copy", &[ValType::I32; 4], &[]);
    assert_eq!(
        check_imports(&[missing], &avail),
        DeployCheck::UnknownFunction("copy".into())
    );
    let mismatch = need("syscall", "publish", &[ValType::I64], &[]);
    assert_eq!(
        check_imports(&[mismatch], &avail),
        DeployCheck::SignatureMismatch("publish".into())
    );
    assert_eq!(
        check_imports(&[publish.clone(), publish], &avail),
        DeployCheck::DuplicateImport("publish".into())
    );
}
