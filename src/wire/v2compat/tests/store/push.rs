use super::{PMeta, Push, R, Req, Resp, Un};
impl Push for R {
    fn push_subscribe(&self, _: &Req) -> Resp {
        Un.health()
    }
    fn push_update(&self, _: &str, _: &Req) -> Resp {
        Un.health()
    }
    fn push_delete(&self, _: &str, _: &Req) -> Resp {
        Un.health()
    }
}
