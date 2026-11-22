use std::{collections::HashMap, net::IpAddr};
use webpush_sender::{error::IoFailure, push::Rng, sub::Resolver};

pub struct FakeResolver {
    pub answers: HashMap<String, Vec<IpAddr>>,
}

impl Resolver for FakeResolver {
    fn resolve(&self, host: &str) -> core::result::Result<Vec<IpAddr>, IoFailure> {
        self.answers.get(host).cloned().ok_or(IoFailure)
    }
}

pub fn one(ip: &str) -> FakeResolver {
    resolver(&[("push.example", ip)])
}

pub fn resolver(entries: &[(&str, &str)]) -> FakeResolver {
    let mut answers = HashMap::new();
    for (host, ip) in entries {
        answers.insert(host.to_string(), vec![ip.parse::<IpAddr>().unwrap()]);
    }
    FakeResolver { answers }
}

pub struct FixedRng {
    pub seed: [u8; 32],
}

impl Rng for FixedRng {
    fn fill(&self, out: &mut [u8]) {
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = self.seed[i % self.seed.len()];
        }
    }
}
