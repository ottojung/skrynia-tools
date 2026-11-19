#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ack,
    Retry,
    Gone,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Pending {
    pub sub_id: String,
    pub endpoint: String,
    pub attempts: u32,
    pub next_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub sub_id: String,
    pub endpoint: String,
}
