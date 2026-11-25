use super::Rng;
use std::io::Read;

pub struct OsRng;

impl Rng for OsRng {
    fn fill(&self, out: &mut [u8]) {
        let mut file = std::fs::File::open("/dev/urandom").expect("os rng unavailable");
        file.read_exact(out).expect("os rng read failed");
    }
}
