use super::{Callback, InvokeBackend};
use crate::durable;
use skrynia_tools::wire::v2compat::{Admin, CompatResult};

impl<I: Callback> Admin for InvokeBackend<'_, I> {
    fn grant_create(&self, token: &str, body: &[u8]) -> CompatResult<Vec<u8>> {
        durable::admin::create(self.invoke, token, body)
    }
    fn grant_get(&self, id: &str, token: &str) -> CompatResult<Vec<u8>> {
        durable::admin::get(self.invoke, id, token)
    }
    fn grant_issue_user(&self, id: &str, token: &str) -> CompatResult<Vec<u8>> {
        durable::admin::issue(self.invoke, id, token)
    }
    fn grant_revoke_user(&self, id: &str, user: &str, token: &str) -> CompatResult<()> {
        durable::admin::revoke_user(self.invoke, id, user, token)
    }
    fn grant_revoke(&self, id: &str, token: &str) -> CompatResult<()> {
        durable::admin::grant(self.invoke, id, token)
    }
}
