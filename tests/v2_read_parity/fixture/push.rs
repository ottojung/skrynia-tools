use super::MemoryStore;
use skrynia_tools::wire::v2compat::Meta;
impl skrynia_tools::wire::v2compat::Push for MemoryStore {
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
