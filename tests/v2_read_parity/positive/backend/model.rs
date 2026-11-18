use std::{collections::HashMap, sync::Mutex};
pub struct RootStore {
    pub(super) token: String,
    pub(super) data: Mutex<HashMap<String, Vec<u8>>>,
}
impl RootStore {
    pub fn new(token: String) -> Self {
        Self {
            token,
            data: Mutex::new(HashMap::new()),
        }
    }
}
