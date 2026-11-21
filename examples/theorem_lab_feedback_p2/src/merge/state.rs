use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub version: u8,
    pub updated_at: Option<String>,
    pub html_fragments: BTreeMap<String, String>,
    pub feedback: Vec<Entry>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub comment: String,
    #[serde(default)]
    pub page_url: String,
    #[serde(default)]
    pub location: Option<serde_json::Value>,
    #[serde(default)]
    pub anchor: Option<serde_json::Value>,
    #[serde(default)]
    pub html_chain: Vec<String>,
}
impl State {
    pub fn empty() -> Self {
        Self {
            version: 1,
            updated_at: None,
            html_fragments: BTreeMap::new(),
            feedback: Vec::new(),
        }
    }
}
