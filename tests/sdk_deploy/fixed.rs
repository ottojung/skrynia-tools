use skrynia_tools::sdk::{DeployCheck, ImportNeed, MODULE, ValType, check_imports, fixed};

fn need(module: &str, name: &str, params: &[ValType], results: &[ValType]) -> ImportNeed {
    ImportNeed {
        module: module.into(),
        name: name.into(),
        params: params.to_vec(),
        results: results.to_vec(),
    }
}

#[test]
fn deployment_checks_fixed_namespace_and_exact_signatures() {
    let sig = fixed("read").unwrap();
    let read = need(MODULE, sig.name, &sig.params, &sig.results);
    assert_eq!(check_imports(&[read], &[]), DeployCheck::Ready);
    let retired = need("skrynia", "read1", &[ValType::I32], &[ValType::I32]);
    assert_eq!(
        check_imports(&[retired], &[]),
        DeployCheck::ForbiddenModule("skrynia".into())
    );
    let wrong = need(MODULE, "read", &[ValType::I32], &[ValType::I64]);
    assert_eq!(
        check_imports(&[wrong], &[]),
        DeployCheck::SignatureMismatch("read".into())
    );
    let unknown = need(MODULE, "copy", &[ValType::I32; 4], &[]);
    assert_eq!(
        check_imports(&[unknown], &[]),
        DeployCheck::UnknownFunction("copy".into())
    );
}
