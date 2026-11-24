/// Bounded per-cycle accounting; no push payload or endpoint is exposed.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Report {
    pub hints: usize,
    pub delivered: usize,
    pub acked: usize,
    pub skipped: usize,
}
