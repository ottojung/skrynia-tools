use super::*;
pub fn run(
    server: &Server,
    backend: &Backend,
    old: (&str, &str),
    new: (&str, &str),
) -> Result<(), String> {
    let users = compare::at(
        &server.url,
        backend,
        "POST",
        Endpoint {
            path: &format!("/grants/{}/users", old.0),
            token: old.1,
        },
        Endpoint {
            path: &format!("/grants/{}/users", new.0),
            token: new.1,
        },
        b"",
    )?;
    if users.0.0 != 200 || users.1.0 != 200 {
        return Err(format!("issue status {}", users.0.0));
    }
    let old_user = users.0.1["userId"].as_str().ok_or("old user id")?;
    let new_user = users.1.1["userId"].as_str().ok_or("new user id")?;
    let deleted = compare::at(
        &server.url,
        backend,
        "DELETE",
        Endpoint {
            path: &format!("/grants/{}/users/{old_user}", old.0),
            token: old.1,
        },
        Endpoint {
            path: &format!("/grants/{}/users/{new_user}", new.0),
            token: new.1,
        },
        b"",
    )?;
    if deleted.0.0 != deleted.1.0 {
        return Err(format!("user revoke {deleted:?}"));
    }
    let mut old_body = deleted.0.1;
    old_body["grantId"] = deleted.1.1["grantId"].clone();
    old_body["userId"] = deleted.1.1["userId"].clone();
    compare::equal(((deleted.0.0, old_body), deleted.1))
}
