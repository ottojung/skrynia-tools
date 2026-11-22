//! One bounded consumption cycle. A hint only wakes the sender; endpoint
//! material is re-read after the hint (never treated as commit proof).
use super::{Pump, Report, Result, process_seq, seqs};

impl<P, O, R, G, C> Pump<'_, P, O, R, G, C>
where
    P: super::P1Transport,
    O: super::PushTransport,
    R: super::Resolver,
    G: super::Rng,
    C: super::Clock,
{
    pub fn run_once(&mut self) -> Result<Report> {
        let raw = self.p1.hint_list()?;
        let seqs = seqs::parse(&raw)?;
        let mut report = Report {
            hints: seqs.len(),
            ..Report::default()
        };
        let mut failure = None;
        for seq in seqs {
            if self.ledger.seen(seq) {
                report.skipped += 1;
                continue;
            }
            match process_seq(self, seq) {
                Ok(sent) => {
                    // All structurally valid subscriptions were handled. ACK even
                    // when unsafe records were dropped: otherwise one bad sub
                    // would starve every later seq forever with no ACK.
                    self.p1.hint_ack(seq)?;
                    self.ledger.mark(seq);
                    report.delivered += sent;
                    report.acked += 1;
                }
                Err(e) => {
                    // A deliverable subscription failed: no ACK for this seq,
                    // but keep draining later hints before surfacing the error.
                    failure = Some(e);
                }
            }
        }
        match failure {
            Some(e) => Err(e),
            None => Ok(report),
        }
    }
}
