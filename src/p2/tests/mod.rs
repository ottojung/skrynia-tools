//! Host-side tests with real WASM guests compiled from WAT.

mod budget;
mod callback;
mod frames;
mod limits;
mod reject;
mod replay;
mod roundtrip;
mod trap;

pub(crate) type Binding = crate::p2::Binding;
pub(crate) type Error = crate::p2::Error;
pub(crate) type Interpreter = crate::p2::Interpreter;
pub(crate) type Limits = crate::p2::Limits;

pub(crate) fn module(wat_source: &str) -> Vec<u8> {
    wat::parse_str(wat_source).expect("fixture WAT must compile")
}

pub(crate) const P0_IMPORT: &str =
    "(import \"p0\" \"invoke\" (func $invoke (param i32 i32 i32) (result i64)))";
