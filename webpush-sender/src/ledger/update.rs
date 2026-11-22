//! Decisions and durable state transitions. A failed save never advances
//! local state; only exact (binding, seq) matches suppress work.
use super::model::{Record, State};
use super::{Decision, DurableLedger, Error, MAX_HINTS, Result, encode};

impl DurableLedger {
    /// Exact matches only. An unseen seq at or below the maximum is rejected:
    /// seq reuse/reset fails closed rather than suppressing a valid hint.
    pub fn decide(&self, seq: u64) -> Result<Decision> {
        if let Some(record) = self.records.iter().find(|r| r.seq == seq) {
            return Ok(match record.state {
                State::Delivered => Decision::Delivered,
                State::Acked => Decision::Acked,
            });
        }
        if self.records.iter().any(|r| r.seq >= seq) {
            return Err(Error::Wire("ledger-reuse"));
        }
        Ok(Decision::Fresh)
    }

    /// Persist DELIVERED before the ACK RPC. At cap, evict the oldest record:
    /// the ledger stays bounded and never silently stops recording.
    pub fn mark_delivered(&mut self, seq: u64) -> Result<()> {
        let mut next = self.records.clone();
        if next.len() >= MAX_HINTS {
            next.remove(0);
        }
        next.push(Record {
            seq,
            state: State::Delivered,
        });
        self.store.save(&encode(&self.binding, MAX_HINTS, &next))?;
        self.records = next;
        Ok(())
    }

    /// Persist ACKED after an accepted ACK. On save failure the local state
    /// stays DELIVERED, so replay re-ACKs instead of re-sending.
    pub fn mark_acked(&mut self, seq: u64) -> Result<()> {
        let mut next = self.records.clone();
        let record = next
            .iter_mut()
            .find(|r| r.seq == seq)
            .ok_or(Error::Wire("ledger-missing"))?;
        record.state = State::Acked;
        self.store.save(&encode(&self.binding, MAX_HINTS, &next))?;
        self.records = next;
        Ok(())
    }
}
