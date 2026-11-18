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
impl CompatBackend for RootStore {}
