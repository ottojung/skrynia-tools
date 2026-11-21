use super::CompatResult;
pub trait Admin {
    fn grant_create(&self, _token: &str, _body: &[u8]) -> CompatResult<Vec<u8>> {
        Err(super::CompatError::new(501, "not_implemented"))
    }
    fn grant_get(&self, _id: &str, _token: &str) -> CompatResult<Vec<u8>> {
        Err(super::CompatError::new(501, "not_implemented"))
    }
    fn grant_issue_user(&self, _id: &str, _token: &str) -> CompatResult<Vec<u8>> {
        Err(super::CompatError::new(501, "not_implemented"))
    }
    fn grant_revoke_user(&self, _id: &str, _user: &str, _token: &str) -> CompatResult<()> {
        Err(super::CompatError::new(501, "not_implemented"))
    }
    fn grant_revoke(&self, _id: &str, _token: &str) -> CompatResult<()> {
        Err(super::CompatError::new(501, "not_implemented"))
    }
}
