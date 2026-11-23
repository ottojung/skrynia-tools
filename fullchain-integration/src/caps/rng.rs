//! Fixed randomness: makes VAPID salt, ephemeral key and ciphertext reproducible.
use webpush_sender::push::Rng;

pub struct FixedRng;

impl Rng for FixedRng {
    fn fill(&self, out: &mut [u8]) {
        for (i, b) in out.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(31).wrapping_add(4);
        }
    }
}
