use super::CompatResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object {
    pub key: String,
    pub bytes: Vec<u8>,
}
pub enum ObjectRead {
    Missing,
    Found(Object),
}
pub trait Store {
    fn read_object(&self, _key: &str) -> CompatResult<ObjectRead> {
        Ok(ObjectRead::Missing)
    }
}
