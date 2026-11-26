//! Fail-closed boundary validation before any instantiation.
//!
//! Exactly one import is admitted: the `p0.invoke(ctx, op, value) -> i64`
//! supplied by P1. Anything else (a second import, a `skrynia.*` name, a
//! pointer-shaped signature or a wrong `_start`) is rejected before execution.

use super::binding::Binding;
use super::error::Error;
use wasmi::{ExternType, Module, ValType};

pub(crate) fn check(module: &Module, binding: &Binding) -> Result<(), Error> {
    let mut imports = module.imports();
    let Some(only) = imports.next() else {
        return Err(Error::MissingImport);
    };
    if imports.next().is_some() {
        return Err(Error::ExtraImport);
    }
    if only.module() != binding.module() || only.name() != binding.function() {
        return Err(Error::WrongImport);
    }
    let Some(func) = only.ty().func() else {
        return Err(Error::WrongImport);
    };
    let params = [ValType::I32, ValType::I32, ValType::I32];
    let results = [ValType::I64];
    if func.params() != params.as_slice() || func.results() != results.as_slice() {
        return Err(Error::WrongImport);
    }
    match module.get_export("_start") {
        Some(ExternType::Func(start)) => {
            if start.params().is_empty() && start.results().is_empty() {
                Ok(())
            } else {
                Err(Error::WrongStart)
            }
        }
        _ => Err(Error::MissingStart),
    }
}
