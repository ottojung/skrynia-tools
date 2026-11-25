use super::{Error, Result};
use serde_json::Value;

pub(super) fn str_field<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or(Error::Config("config-field"))
}

pub(super) fn u64_field(v: &Value, key: &str) -> Result<u64> {
    v.get(key)
        .and_then(Value::as_u64)
        .ok_or(Error::Config("config-field"))
}

pub(super) fn u32_field(v: &Value, key: &str) -> Result<u32> {
    let n = u64_field(v, key)?;
    u32::try_from(n).map_err(|_| Error::Config("config-field"))
}

pub(super) fn u16_field(v: &Value, key: &str) -> Result<u16> {
    let n = u64_field(v, key)?;
    u16::try_from(n).map_err(|_| Error::Config("config-field"))
}

pub(super) fn usize_field(v: &Value, key: &str) -> Result<usize> {
    let n = u64_field(v, key)?;
    usize::try_from(n).map_err(|_| Error::Config("config-field"))
}
