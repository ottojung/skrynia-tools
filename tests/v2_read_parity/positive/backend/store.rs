use super::*;
impl Store for RootStore {
    fn read_object(&self, key: &str) -> CompatResult<ObjectRead> {
        let map = self.data.lock().unwrap();
        Ok(match map.get(key) {
            Some(bytes) => ObjectRead::Found(Object {
                key: key.into(),
                bytes: bytes.clone(),
            }),
            None => ObjectRead::Missing,
        })
    }
    fn mutate_object(&self, req: Mutation<'_>) -> CompatResult<bool> {
        super::mutator::apply(self, req)
    }
}

impl skrynia_tools::wire::v2compat::Push for RootStore {
    fn push_subscribe(
        &self,
        _req: &skrynia_tools::wire::v2compat::Request,
    ) -> skrynia_tools::wire::v2compat::Response {
        skrynia_tools::wire::v2compat::Unavailable.health()
    }
    fn push_update(
        &self,
        _id: &str,
        _req: &skrynia_tools::wire::v2compat::Request,
    ) -> skrynia_tools::wire::v2compat::Response {
        skrynia_tools::wire::v2compat::Unavailable.health()
    }
    fn push_delete(
        &self,
        _id: &str,
        _req: &skrynia_tools::wire::v2compat::Request,
    ) -> skrynia_tools::wire::v2compat::Response {
        skrynia_tools::wire::v2compat::Unavailable.health()
    }
}

impl CompatBackend for RootStore {}
