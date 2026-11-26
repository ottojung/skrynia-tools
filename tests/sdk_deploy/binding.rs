use skrynia_tools::sdk::{BindingError, NO_FUNCTION, Signature, ValType, check_binding};

#[test]
fn eval_binding_is_exact_and_has_no_implicit_import() {
    let invoke = Signature::new("invoke", &[ValType::I32, ValType::I64], &[ValType::I32]);
    let table = [invoke];
    assert!(check_binding(0, 11, 22, &table, Some(&table[0])).is_ok());
    assert_eq!(check_binding(NO_FUNCTION, 0, 0, &table, None), Ok(()));
    assert_eq!(
        check_binding(NO_FUNCTION, 1, 0, &table, None),
        Err(BindingError::NoFunctionWithHandles)
    );
    assert_eq!(
        check_binding(3, 0, 0, &table, Some(&table[0])),
        Err(BindingError::SelectorOutOfRange)
    );
    assert_eq!(
        check_binding(0, 0, 0, &table, None),
        Err(BindingError::FunctionWithoutImport)
    );
    let wrong = Signature::new("invoke", &[ValType::I64, ValType::I32], &[ValType::I32]);
    assert_eq!(
        check_binding(0, 0, 0, &table, Some(&wrong)),
        Err(BindingError::TypeMismatch)
    );
}
