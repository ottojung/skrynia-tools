use super::*;
pub fn run(
    server: &Server,
    backend: &Backend,
    old: (&str, &str),
    new: (&str, &str),
) -> Result<(), String> {
    let removed = compare::at(
        &server.url,
        backend,
        "DELETE",
        Endpoint {
            path: &format!("/grants/{}", old.0),
            token: old.1,
        },
        Endpoint {
            path: &format!("/grants/{}", new.0),
            token: new.1,
        },
        b"",
    )?;
    let mut old_body = removed.0.1;
    old_body["grantId"] = removed.1.1["grantId"].clone();
    compare::equal(((removed.0.0, old_body), removed.1))
}
