//! One bounded consumption cycle. A hint only wakes the sender; endpoint
//! material is re-read after the hint (never treated as commit proof).
use super::{Decision, Pump, Report, Result, process_seq, seqs};

impl<P, O, R, G, C> Pump<'_, P, O, R, G, C>
where
    P: super::P1Transport,
    O: super::PushTransport,
    R: super::Resolver,
    G: super::Rng,
    C: super::Clock,
{
    pub fn run_once(&mut self) -> Result<Report> {
        let seqs = seqs::parse(&self.p1.hint_list()?)?;
        let mut report = Report {
            hints: seqs.len(),
            ..Report::default()
        };
        let mut failure = None;
        for seq in seqs {
            match self.ledger.decide(seq)? {
                Decision::Acked => {
                    report.skipped += 1;
                    continue;
                }
                Decision::Delivered => {
                    // Marker persisted but the ACK reply may have been lost:
                    // re-ACK idempotently instead of re-sending or skipping.
                    self.p1.hint_ack(seq)?;
                    self.ledger.mark_acked(seq)?;
                    report.acked += 1;
                    continue;
                }
                Decision::Fresh => {}
            }
            match process_seq(self, seq) {
                Ok(sent) => {
                    self.ledger.mark_delivered(seq)?; // durable BEFORE the ACK
                    self.p1.hint_ack(seq)?;
                    self.ledger.mark_acked(seq)?;
                    report.delivered += sent;
                    report.acked += 1;
                }
                Err(e) => failure = Some(e),
            }
        }
        match failure {
            Some(e) => Err(e),
            None => Ok(report),
        }
    }
}
