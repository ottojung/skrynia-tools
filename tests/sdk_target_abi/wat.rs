use skrynia_tools::sdk::{FFI_MODULE, MODULE};

const P0_CHILD: &str =
    include_str!("../../v3-trusted-edge-gateway/tests/support/kernel/fixtures/p0_child_echo.wat");
const P2_ECHO: &str =
    include_str!("../../v3-trusted-edge-gateway/tests/support/kernel/fixtures/p2_echo.wat");

#[test]
fn hand_authored_child_wat_keeps_only_userland_invoke() {
    for source in [P0_CHILD, P2_ECHO] {
        let engine = wasmi::Engine::default();
        let module = wasmi::Module::new(&engine, wat::parse_str(source).unwrap()).unwrap();
        let imports: Vec<_> = module.imports().map(|i| (i.module(), i.name())).collect();
        assert_eq!(imports, vec![("p0", "invoke")]);
        assert!(module.exports().any(|e| e.name() == "_start"));
        assert!(
            !imports
                .iter()
                .any(|(m, _)| *m == MODULE || *m == FFI_MODULE)
        );
    }
    let engine = wasmi::Engine::default();
    let module = wasmi::Module::new(&engine, wat::parse_str(P0_CHILD).unwrap()).unwrap();
    assert!(module.exports().any(|e| e.name() == "memory"));
}
