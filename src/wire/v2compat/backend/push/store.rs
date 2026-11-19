use super::{CompatError, Keys, Record};
#[allow(dead_code)]
pub trait Store {
    fn find_dedup(&self, grant: &str, r: &Record) -> Option<Record>;
    fn insert(&mut self, r: Record) -> Result<(), CompatError>;
    fn get(&self, id: &str) -> Option<Record>;
    fn update_target(&mut self, id: &str, endpoint: String, keys: Keys) -> Result<(), CompatError>;
    fn set_cap_hash(&mut self, id: &str, cap_hash: String) -> Result<(), CompatError>;
    fn remove(&mut self, id: &str) -> Result<(), CompatError>;
    fn remove_by_grant(&mut self, grant_id: &str);
    fn all(&self) -> Vec<Record>;
}
