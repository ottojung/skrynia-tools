//! Per-sequence step inside one bounded consumption cycle.
use super::{Decision, Error, Pump, Report, keep_worse, process_seq};

impl<P, O, R, G, C> Pump<'_, P, O, R, G, C>
where
    P: super::P1Transport,
    O: super::PushTransport,
    R: super::Resolver,
    G: super::Rng,
    C: super::Clock,
{
    pub(super) fn step_seq(&mut self, seq: u64, report: &mut Report, failure: &mut Option<Error>) {
        // An unseen seq at/below the recorded max is an out-of-order unacked
        // hint. Isolate this soft signal so later seqs are not starved;
        // corruption/quota still fail closed in open() and mark_*().
        let decision = match self.ledger.decide(seq) {
            Err(Error::Wire("ledger-reuse")) => Decision::Fresh,
            Err(e) => return keep_worse(failure, e),
            Ok(d) => d,
        };
        match decision {
            Decision::Acked => report.skipped += 1,
            Decision::Delivered => self.reack(seq, report, failure),
            Decision::Fresh => self.deliver_seq(seq, report, failure),
        }
    }

    fn reack(&mut self, seq: u64, report: &mut Report, failure: &mut Option<Error>) {
        // Marker persisted but the ACK reply may have been lost: re-ACK
        // idempotently instead of re-sending or skipping.
        match self
            .p1
            .hint_ack(seq)
            .and_then(|_| self.ledger.mark_acked(seq))
        {
            Ok(()) => report.acked += 1,
            Err(e) => keep_worse(failure, e),
        }
    }

    fn deliver_seq(&mut self, seq: u64, report: &mut Report, failure: &mut Option<Error>) {
        match process_seq(self, seq) {
            Ok(sent) => match self
                .ledger
                .mark_delivered(seq)
                .and_then(|_| self.p1.hint_ack(seq))
                .and_then(|_| self.ledger.mark_acked(seq))
            {
                Ok(()) => {
                    report.delivered += sent;
                    report.acked += 1;
                }
                Err(e) => keep_worse(failure, e),
            },
            Err(e) => keep_worse(failure, e),
        }
    }
}
