//! Loopback-only trusted-edge HTTP adapter for P1 opcodes 20..=23.
//! The bearer here is an independent, app-scoped edge capability. This module
//! never sees, stores or forwards the private operator/P0 token.
mod call;
#[cfg(test)]
mod cases;
#[cfg(test)]
mod deadline;
mod endpoint;
#[cfg(test)]
mod guards;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod mock_io;
#[cfg(test)]
mod mock_reply;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) use super::{Client, Config, P1Transport};
use super::{Error, IoFailure, Result};
pub use endpoint::Edge;

pub struct HttpP1 {
    edge: Edge,
    client: reqwest::blocking::Client,
}

impl HttpP1 {
    pub fn new(edge: Edge) -> Result<Self> {
        edge.validate()?;
        Ok(Self {
            client: call::client(&edge)?,
            edge,
        })
    }

    pub fn edge(&self) -> &Edge {
        &self.edge
    }
}

impl super::P1Transport for HttpP1 {
    fn call(&self, req: &[u8]) -> core::result::Result<Vec<u8>, IoFailure> {
        call::post(&self.client, &self.edge, req)
    }
}

#[cfg(test)]
pub(super) use mock::{MockEdge, Reply};
#[cfg(test)]
pub(super) use mock_io::Seen;
