//! Transient P1 Busy must not mask a later permanent auth rejection.
use super::{
    FixedRng, ManualClock, ScriptedTransport, config, frame, res, resolver, sub_json, vapid_key,
};
use std::cell::Cell;
use webpush_sender::Error;
use webpush_sender::p1::{P1Failure, P1Transport};

struct BusyFirst {
    subs: Vec<u8>,
    reads: Cell<usize>,
}

impl P1Transport for BusyFirst {
    fn call(&self, req: &[u8]) -> core::result::Result<Vec<u8>, P1Failure> {
        let body = match req.get(1).copied().unwrap_or(0) {
            20 => {
                let mut b = 2u32.to_le_bytes().to_vec();
                b.extend_from_slice(&1u64.to_le_bytes());
                b.extend_from_slice(&2u64.to_le_bytes());
                b
            }
            21 => {
                self.reads.set(self.reads.get() + 1);
                if self.reads.get() == 1 {
                    return Ok(res(1, &[11]));
                }
                [0x5au8; 32].to_vec()
            }
            22 => Vec::new(),
            23 => self.subs.clone(),
            _ => return Err(P1Failure::Io),
        };
        Ok(res(0, &body))
    }
}

pub fn busy_does_not_mask_later_auth() {
    let (cfg, key, rng, clock) = (
        config(),
        vapid_key(),
        FixedRng { seed: [4u8; 32] },
        ManualClock::new(0),
    );
    let p1 = BusyFirst {
        subs: frame(&[sub_json("https://a.example/p")]),
        reads: Cell::new(0),
    };
    let r = resolver(&[("a.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![401]);
    let mut pump = webpush_sender::Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert_eq!(pump.run_once(), Err(Error::Auth("push")));
    assert_eq!(out.calls.borrow().len(), 1);
}
