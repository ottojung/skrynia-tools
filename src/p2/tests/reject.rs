use super::{Error, Interpreter};
use super::{P0_IMPORT, module};

fn check(wat_source: &str) -> Error {
    Interpreter::new()
        .evaluate(&module(wat_source), &[], None)
        .unwrap_err()
}

#[test]
fn missing_import_is_rejected() {
    assert_eq!(
        check("(module (func (export \"_start\")))"),
        Error::MissingImport
    );
}

#[test]
fn extra_import_is_rejected() {
    let src = format!(
        "(module {P0_IMPORT} (import \"other\" \"invoke\" (func (param i32 i32 i32) (result i64))) (func (export \"_start\")))"
    );
    assert_eq!(check(&src), Error::ExtraImport);
}

#[test]
fn wrong_import_pair_or_signature_is_rejected() {
    let wrong_module = "(module (import \"skrynia\" \"invoke\" (func (param i32 i32 i32) (result i64))) (func (export \"_start\")))";
    assert_eq!(check(wrong_module), Error::WrongImport);
    let wrong_types = "(module (import \"p0\" \"invoke\" (func (param i32 i32 i64) (result i64))) (func (export \"_start\")))";
    assert_eq!(check(wrong_types), Error::WrongImport);
}

#[test]
fn missing_or_wrong_start_is_rejected() {
    let no_start = format!("(module {P0_IMPORT})");
    assert_eq!(check(&no_start), Error::MissingStart);
    let wrong_start = format!("(module {P0_IMPORT} (func (export \"_start\") (param i32)))");
    assert_eq!(check(&wrong_start), Error::WrongStart);
}

#[test]
fn invalid_bytes_are_rejected() {
    let out = Interpreter::new().evaluate(b"not wasm", &[], None);
    assert_eq!(out, Err(Error::InvalidWasm));
}
