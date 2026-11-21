use super::gateway::Gateway;
use std::time::Duration;

pub fn assert_no_leak(gw: &mut Gateway, token: [u8; 32], cap_a: &str, cap_b: &str) {
    std::thread::sleep(Duration::from_millis(50));
    let stderr = gw.stderr_text();
    let token_hex: String = token.iter().map(|b| format!("{b:02x}")).collect();
    assert!(!stderr.contains(cap_a) && !stderr.contains(cap_b));
    assert!(!stderr.contains(&token_hex));
}
