//! One bounded consumption cycle. A hint only wakes the sender; endpoint
//! material is re-read after the hint (never treated as commit proof).
use super::{Pump, Report, Result, fatal, seqs};

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
            // Once a fatal class is recorded, stop: the operator loop must
            // exit rather than retry a permanent auth/config/quota failure.
            if failure.as_ref().is_some_and(fatal) {
                break;
            }
            self.step_seq(seq, &mut report, &mut failure);
        }
        failure.map_or(Ok(report), Err)
    }
}
