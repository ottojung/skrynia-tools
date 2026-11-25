//! High-level client. All I/O is injected; malformed responses fail closed.
use super::{Config, P1Failure, Result, decode, hint_ack, hint_list, hint_read, sub_fetch};

pub trait P1Transport {
    fn call(&self, req: &[u8]) -> core::result::Result<Vec<u8>, P1Failure>;
}

pub struct Client<'a, T: P1Transport> {
    cfg: &'a Config,
    io: &'a T,
}

impl<'a, T: P1Transport> Client<'a, T> {
    pub fn new(cfg: &'a Config, io: &'a T) -> Result<Self> {
        cfg.validate()?;
        Ok(Self { cfg, io })
    }

    fn call(&self, req: Vec<u8>) -> Result<Vec<u8>> {
        let raw = self.io.call(&req).map_err(|f| match f {
            P1Failure::Auth => super::Error::Auth("edge"),
            P1Failure::Reject => super::Error::Transport("status"),
            P1Failure::Io => super::Error::Transport("p1"),
        })?;
        decode(&raw)
    }

    pub fn hint_list(&self) -> Result<Vec<u8>> {
        let req = hint_list(&self.cfg.sender_cap, &self.cfg.ns, &self.cfg.key)?;
        self.call(req)
    }

    pub fn hint_read(&self, seq: u64) -> Result<Vec<u8>> {
        let req = hint_read(&self.cfg.sender_cap, &self.cfg.ns, &self.cfg.key, seq)?;
        self.call(req)
    }

    pub fn hint_ack(&self, seq: u64) -> Result<()> {
        let req = hint_ack(&self.cfg.sender_cap, &self.cfg.ns, &self.cfg.key, seq)?;
        self.call(req).map(|_| ())
    }

    pub fn sub_fetch(&self) -> Result<Vec<u8>> {
        let req = sub_fetch(&self.cfg.sender_cap, &self.cfg.ns, &self.cfg.key)?;
        self.call(req)
    }
}
