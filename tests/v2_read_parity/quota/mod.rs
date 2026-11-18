mod cases;
mod model;
mod setup;
use super::process::Server;
pub(super) use super::{RootStore, compare};
pub fn verify(server: &Server) -> Result<(), String> {
    let token = setup::token(server)?;
    let store = RootStore::with_authority(model::authority(&token));
    cases::run(&server.url, &token, &store)
}
