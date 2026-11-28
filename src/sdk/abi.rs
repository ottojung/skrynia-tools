//! Fixed machine ABI inventory (21 imports; bulk I/O over linear memory).
use super::typed::{FixedSignature, ValType};

/// Namespace of every fixed import in ABI 3.1. The retired `skrynia`
/// namespace is never linked.
pub const MODULE: &str = "skrynia_v3_1";
/// Namespace of operator-defined named FFI imports; root-only.
pub const FFI_MODULE: &str = "syscall";

const TABLE: [(&str, &str, &str); 21] = [
    ("open_ram", "", "i32"),
    ("open_disk", "i64,i64,i32", "i32"),
    ("close", "i32", "i32"),
    ("read", "i32,i64,i64", "i64"),
    ("write", "i32,i64,i64", "i64"),
    ("read1", "i32", "i32"),
    ("write1", "i32,i32", "i32"),
    ("seek", "i32,i64", "i64"),
    ("truncate", "i32,i64", "i32"),
    ("sync", "i32", "i32"),
    ("mkdir", "i64,i64,i32", "i32"),
    ("exists", "i64,i64", "i32"),
    ("is_dir", "i64,i64", "i32"),
    ("delete", "i64,i64", "i32"),
    ("rename", "i64,i64,i64,i64", "i32"),
    ("open_directory", "i64,i64", "i32"),
    ("next_entry", "i32,i64,i64", "i32"),
    ("cas1", "i32,i64,i32,i32", "i32"),
    ("now", "", "i64"),
    ("secure_random_bits", "i64,i64", "i32"),
    ("eval", "i64,i64,i32,i32,i32", "i64"),
];

fn types(text: &str) -> Vec<ValType> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split(',')
            .map(|item| ValType::parse(item).expect("fixed ABI type"))
            .collect()
    }
}

/// Exact name and ordered integer types of one fixed import, if it exists.
pub fn fixed(name: &str) -> Option<FixedSignature> {
    TABLE
        .iter()
        .find(|entry| entry.0 == name)
        .map(|entry| FixedSignature {
            name: entry.0,
            params: types(entry.1),
            results: types(entry.2),
        })
}

/// Every fixed import name; optional named FFI imports are discovered.
pub fn names() -> impl Iterator<Item = &'static str> {
    TABLE.iter().map(|entry| entry.0)
}
