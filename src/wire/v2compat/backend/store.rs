use super::CompatResult;
use crate::wire::v2compat::types::Object;

pub enum ObjectRead {
    Missing,
    Found(Object),
}

pub trait Store {
    fn read_object(&self, _key: &str) -> CompatResult<ObjectRead> {
        Ok(ObjectRead::Missing)
    }
}
