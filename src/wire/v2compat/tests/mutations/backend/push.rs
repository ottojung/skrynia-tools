use super::Memory;
use super::{PMeta, Push, Req, Resp, Un};
impl Push for Memory {
    fn push_subscribe(&self, _req: &Req) -> Resp {
        Un.health()
    }
    fn push_update(&self, _id: &str, _req: &Req) -> Resp {
        Un.health()
    }
    fn push_delete(&self, _id: &str, _req: &Req) -> Resp {
        Un.health()
    }
}
