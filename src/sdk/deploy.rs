//! Deployment-time import selection: no unresolved optional imports.
use super::typed::{Signature, ValType};

/// One WASM import a module needs, with its exact ordered integer types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportNeed {
    pub name: String,
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

impl ImportNeed {
    pub fn new(name: &str, params: &[ValType], results: &[ValType]) -> Self {
        Self {
            name: name.to_owned(),
            params: params.to_vec(),
            results: results.to_vec(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeployCheck {
    Ready,
    UnknownFunction(String),
    SignatureMismatch(String),
    DuplicateImport(String),
}

/// Select a module only when every import is present and matches exactly.
///
/// An omitted optional function (including copy on an HTTP provider) is
/// UnknownFunction, never a reason to bind a fallback after execution starts.
pub fn check_imports(needs: &[ImportNeed], available: &[Signature]) -> DeployCheck {
    for (index, need) in needs.iter().enumerate() {
        if needs[..index]
            .iter()
            .any(|earlier| earlier.name == need.name)
        {
            return DeployCheck::DuplicateImport(need.name.clone());
        }
        let Some(found) = available.iter().find(|sig| sig.name == need.name) else {
            return DeployCheck::UnknownFunction(need.name.clone());
        };
        if found.params != need.params || found.results != need.results {
            return DeployCheck::SignatureMismatch(need.name.clone());
        }
    }
    DeployCheck::Ready
}
