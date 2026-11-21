pub fn max_body() -> usize {
    16 * 1024 * 1024
}
pub fn concurrent() -> usize {
    8
}
pub fn acquire_ms() -> u64 {
    5_000
}
pub fn request_ms() -> u64 {
    120_000
}
pub fn upload_idle_ms() -> u64 {
    10_000
}
pub fn upstream_connect_ms() -> u64 {
    5_000
}
pub fn upstream_read_ms() -> u64 {
    30_000
}
