//! Deployment-time import selection: fixed namespace and exact typed FFI.
use super::abi::{FFI_MODULE, MODULE, fixed};
use super::typed::{Signature, ValType};

/// One WASM import a module needs, with its namespace and exact integer types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportNeed {
    pub module: String,
    pub name: String,
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeployCheck {
    Ready,
    ForbiddenModule(String),
    UnknownFunction(String),
    SignatureMismatch(String),
    DuplicateImport(String),
}

/// Select a module only when every import is present and matches exactly.
/// Fixed imports use the SDK inventory; named FFI imports use discovery.
/// An omitted optional import is UnknownFunction, never a post-start fallback.
pub fn check_imports(needs: &[ImportNeed], available: &[Signature]) -> DeployCheck {
    for (index, need) in needs.iter().enumerate() {
        if needs[..index]
            .iter()
            .any(|e| e.module == need.module && e.name == need.name)
        {
            return DeployCheck::DuplicateImport(need.name.clone());
        }
        if need.module != MODULE && need.module != FFI_MODULE {
            return DeployCheck::ForbiddenModule(need.module.clone());
        }
        let expected = if need.module == MODULE {
            fixed(&need.name).map(|s| (s.params, s.results))
        } else {
            available
                .iter()
                .find(|s| s.name == need.name)
                .map(|s| (s.params.clone(), s.results.clone()))
        };
        let Some((params, results)) = expected else {
            return DeployCheck::UnknownFunction(need.name.clone());
        };
        if params != need.params || results != need.results {
            return DeployCheck::SignatureMismatch(need.name.clone());
        }
    }
    DeployCheck::Ready
}
