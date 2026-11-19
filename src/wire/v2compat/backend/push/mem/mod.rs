use super::{CompatError, Keys, Record, store::Store};
mod mutate;
#[derive(Clone, Debug, Default)]
pub struct Mem {
    pub items: Vec<Record>,
}
impl Store for Mem {
    fn find_dedup(&self, grant: &str, r: &Record) -> Option<Record> {
        self.items
            .iter()
            .find(|x| {
                x.grant_id == grant
                    && x.locations == r.locations
                    && x.events == r.events
                    && x.endpoint == r.endpoint
                    && x.keys == r.keys
            })
            .cloned()
    }
    fn insert(&mut self, r: Record) -> Result<(), CompatError> {
        self.items.push(r);
        Ok(())
    }
    fn get(&self, id: &str) -> Option<Record> {
        self.items.iter().find(|x| x.id == id).cloned()
    }
    fn update_target(&mut self, id: &str, endpoint: String, keys: Keys) -> Result<(), CompatError> {
        mutate::update_target(self, id, endpoint, keys)
    }
    fn set_cap_hash(&mut self, id: &str, cap_hash: String) -> Result<(), CompatError> {
        mutate::set_cap_hash(self, id, cap_hash)
    }
    fn remove(&mut self, id: &str) -> Result<(), CompatError> {
        mutate::remove(self, id)
    }
    fn remove_by_grant(&mut self, grant_id: &str) {
        self.items.retain(|x| x.grant_id != grant_id);
    }
    fn all(&self) -> Vec<Record> {
        self.items.clone()
    }
}
