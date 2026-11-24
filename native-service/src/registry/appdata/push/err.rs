#[derive(Debug)]
pub enum PushErr<D, L> {
    Disk(D),
    Lock(L),
    BadAuth,
    BadKey,
    Missing,
    Busy,
    Quota,
}
