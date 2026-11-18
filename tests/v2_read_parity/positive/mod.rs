mod backend;
mod cases;
use super::process::Server;
use backend::RootStore;
pub fn verify(server: &Server) -> Result<(), String> {
    let token = server.root_user()?;
    let model = RootStore::new(token.clone());
    cases::run(&server.url, &token, &model)
}
