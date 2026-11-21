use skrynia_native_service::try_lock_status;

#[test]
fn kernel_lock_contract_acquired_and_contended() {
    assert_eq!(try_lock_status(1), Ok(true));
    assert_eq!(try_lock_status(0), Ok(false));
}

#[test]
fn kernel_lock_contract_errors_are_not_contention() {
    for code in [-1, -3, -4, -13, 2] {
        assert_eq!(try_lock_status(code), Err(code), "code {code}");
    }
}
