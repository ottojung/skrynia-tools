//! The sender-visible current generation is the durable wake-slot count.
use super::Mem;
use skrynia_native_service::registry::appdata::push;

pub fn hints(d: &Mem, ns: &[u8], key: &[u8]) -> usize {
    let guard = d.0.lock().unwrap();
    if key != push::WAKE_KEY {
        return guard
            .keys()
            .filter(|p| p.starts_with(&push::hints_root("d", ns, key)))
            .count();
    }
    let path = push::wake_slot_path("d", ns, key);
    let Some(bytes) = guard.get(&path) else {
        return 0;
    };
    if bytes.len() != 16 || &bytes[..4] != b"WSL1" || bytes[12] != 1 {
        return 0;
    }
    usize::try_from(u64::from_le_bytes(bytes[4..12].try_into().unwrap())).unwrap_or(usize::MAX)
}
