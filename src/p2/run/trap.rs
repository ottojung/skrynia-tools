use super::Error;

pub(super) fn trap(e: wasmi::Error) -> Error {
    match e.as_trap_code() {
        Some(wasmi::TrapCode::OutOfFuel) => Error::Budget,
        _ => Error::Trap,
    }
}
