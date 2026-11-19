mod grant;
mod users;
use super::{Server, compare, fixture::Backend};
use compare::Endpoint;
pub fn run(
    server: &Server,
    backend: &Backend,
    old: (&str, &str),
    new: (&str, &str),
) -> Result<(), String> {
    users::run(server, backend, old, new)?;
    grant::run(server, backend, old, new)
}
