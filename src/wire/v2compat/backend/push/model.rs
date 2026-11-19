#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Events(pub Vec<String>);
impl Events {
    pub fn default_set() -> Self {
        Self(vec!["create".into(), "replace".into(), "delete".into()])
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Keys {
    pub p256dh: String,
    pub auth: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubReq {
    pub locations: String,
    pub events: Events,
    pub endpoint: String,
    pub keys: Keys,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateReq {
    pub endpoint: String,
    pub keys: Keys,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SubView {
    pub id: String,
    pub capability: String,
    pub deduped: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub id: String,
    pub grant_id: String,
    pub locations: String,
    pub events: Vec<String>,
    pub endpoint: String,
    pub keys: Keys,
    pub cap_hash: String,
}
