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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListPage {
    pub keys: Vec<String>,
    pub next: Option<String>,
}
pub trait Store {
    fn mutate_object(&self, _request: Mutation<'_>) -> CompatResult<bool> {
        Err(super::CompatError::new(501, "not_implemented"))
    }
    fn read_object(&self, _key: &str) -> CompatResult<ObjectRead> {
        Ok(ObjectRead::Missing)
    }
    fn list_objects(
        &self,
        _token: &str,
        _locations: &str,
        _after: Option<&str>,
        _limit: usize,
    ) -> CompatResult<ListPage> {
        Err(super::CompatError::new(501, "not_implemented"))
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
