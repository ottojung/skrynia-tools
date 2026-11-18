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
    fn mutate_object(&self, _request: Mutation<'_>) -> CompatResult<bool> {
        Err(super::CompatError::new(501, "not_implemented"))
    }
    fn read_object(&self, _key: &str) -> CompatResult<ObjectRead> {
        Ok(ObjectRead::Missing)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationKind {
    Create,
    Replace,
    Delete,
}
pub struct Mutation<'a> {
    pub key: &'a str,
    pub kind: MutationKind,
    pub token: &'a str,
    pub expected: Option<&'a str>,
    pub body: &'a [u8],
}
