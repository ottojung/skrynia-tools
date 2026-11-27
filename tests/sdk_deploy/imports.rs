use skrynia_tools::sdk::{DeployCheck, ImportNeed, Signature, ValType, check_imports};

#[test]
fn deployment_rejects_missing_mismatched_and_duplicate_imports() {
    let avail = [Signature::new("publish", &[ValType::I32], &[])];
    let need = ImportNeed::new("publish", &[ValType::I32], &[]);
    assert_eq!(
        check_imports(std::slice::from_ref(&need), &avail),
        DeployCheck::Ready
    );
    let missing = ImportNeed::new("copy", &[ValType::I32; 4], &[]);
    assert_eq!(
        check_imports(&[missing], &avail),
        DeployCheck::UnknownFunction("copy".into())
    );
    let mismatch = ImportNeed::new("publish", &[ValType::I64], &[]);
    assert_eq!(
        check_imports(&[mismatch], &avail),
        DeployCheck::SignatureMismatch("publish".into())
    );
    assert_eq!(
        check_imports(&[need.clone(), need], &avail),
        DeployCheck::DuplicateImport("publish".into())
    );
}
