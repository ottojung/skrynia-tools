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
        let fail = |status, code| CompatError::new(status, code);
        let operation = match req.kind {
            MutationKind::Create => "create",
            MutationKind::Replace => "modify",
            MutationKind::Delete => "delete",
        };
        self.authority.grant_for(req.token, operation, req.key)?;
        let mut map = self.data.lock().unwrap();
        let old = map.get(req.key);
        if req.kind == MutationKind::Create && old.is_some() {
            return Err(fail(409, "already_exists"));
        }
        if req.kind != MutationKind::Create && old.is_none() {
            return Err(fail(404, "not_found"));
        }
        if let Some(expected) = req.expected {
            use sha2::Digest;
            if hex::encode(sha2::Sha256::digest(old.unwrap())) != expected {
                return Err(fail(412, "content_mismatch"));
            }
        }
        match req.kind {
            MutationKind::Delete => {
                map.remove(req.key);
            }
            _ => {
                map.insert(req.key.into(), req.body.to_vec());
            }
        }
        Ok(false)
    }
}
impl CompatBackend for RootStore {}
