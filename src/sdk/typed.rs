//! Integer value types and exact ordered signatures.

/// Name and ordered integer types of a fixed machine import.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedSignature {
    pub name: &'static str,
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

/// WASM integer types admitted by the target ABI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValType {
    I32,
    I64,
}

impl ValType {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
        }
    }
}

/// Ordered parameter and result types of one named function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signature {
    pub name: String,
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

impl Signature {
    pub fn new(name: &str, params: &[ValType], results: &[ValType]) -> Self {
        Self {
            name: name.to_owned(),
            params: params.to_vec(),
            results: results.to_vec(),
        }
    }
}
