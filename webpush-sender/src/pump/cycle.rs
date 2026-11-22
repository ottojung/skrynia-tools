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
        for seq in seqs {
            if self.ledger.seen(seq) {
                report.skipped += 1;
                continue;
            }
            let sent = process_seq(self, seq)?;
            self.p1.hint_ack(seq)?;
            self.ledger.mark(seq);
            report.delivered += sent;
            report.acked += 1;
        }
        Ok(report)
    }
}
