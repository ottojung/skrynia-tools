//! Eval's zero-or-one supplied function and the userland invoke convention.
use super::typed::Signature;

/// Reserved selector for "no supplied function".
pub const NO_FUNCTION: i32 = -1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingError {
    NoFunctionWithHandles,
    FunctionWithoutImport,
    SelectorOutOfRange,
    TypeMismatch,
}

/// Validate one evaluator-supplied function against a child's single import.
///
/// The child's import must match the selected table entry exactly and in
/// order; any additional import stays unresolved and fails linking. The
/// function runs in its originating context: arg0/arg1 are opaque i32
/// context handles, not RAM addresses. Program 0's private token check and
/// `invoke` protocol remain userland, never kernel policy.
pub fn check_binding(
    selector: i32,
    arg0: i32,
    arg1: i32,
    table: &[Signature],
    child_import: Option<&Signature>,
) -> Result<(), BindingError> {
    if selector == NO_FUNCTION {
        if arg0 == 0 && arg1 == 0 && child_import.is_none() {
            return Ok(());
        }
        return Err(BindingError::NoFunctionWithHandles);
    }
    let Ok(index) = usize::try_from(selector) else {
        return Err(BindingError::SelectorOutOfRange);
    };
    let Some(supplied) = table.get(index) else {
        return Err(BindingError::SelectorOutOfRange);
    };
    let Some(child) = child_import else {
        return Err(BindingError::FunctionWithoutImport);
    };
    if child != supplied {
        return Err(BindingError::TypeMismatch);
    }
    Ok(())
}
