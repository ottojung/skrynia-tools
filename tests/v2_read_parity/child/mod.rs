mod cases;
mod model;
mod setup;
use super::process::Server;
pub fn verify(server: &Server) -> Result<(), String> {
    let root_token = server.root_user()?;
    let user = setup::child_user(server, &root_token)?;
    let authority = model::limited(&user);
    cases::verify(&server.url, &user, &authority)
}
