mod run;
mod util;
use super::{CatEval, MemDisk, NoLock, Registry, Rng, auth, decode_res, encode_req, handle};

#[test]
fn wire_lookup_revoke_and_auth() {
    run::run();
}
