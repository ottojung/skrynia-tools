use super::{Server, compare, fixture::Backend};
use serde_json::Value;
pub fn check(server: &Server, b: &Backend, t2: &str, t3: &str) -> Result<(), String> {
    for body in [
        b"{".as_slice(),
        b"{}".as_slice(),
        br#"{"locations":"other/*"}"#.as_slice(),
    ] {
        compare::equal(compare::each(
            &server.url,
            b,
            "POST",
            "/grants",
            t2,
            t3,
            body,
        )?)?;
    }
    compare::equal(compare::each(
        &server.url,
        b,
        "POST",
        "/grants",
        "",
        "",
        b"{}",
    )?)?;
    Ok(())
}
pub fn issue_and_inspect(
    server: &Server,
    b: &Backend,
    v2: (&str, &str),
    v3: (&str, &str),
) -> Result<(), String> {
    let ((s2, user2), (s3, user3)) = compare::at(
        &server.url,
        b,
        "POST",
        compare::Endpoint {
            path: &format!("/grants/{}/users", v2.0),
            token: v2.1,
        },
        compare::Endpoint {
            path: &format!("/grants/{}/users", v3.0),
            token: v3.1,
        },
        b"",
    )?;
    if s2 != 200 || s3 != 200 {
        return Err(format!("issue status {s2} vs {s3}"));
    }
    let t2 = user2["userToken"].as_str().ok_or("missing v2 user token")?;
    let t3 = user3["userToken"].as_str().ok_or("missing v3 user token")?;
    let ((a2, grant2), (a3, grant3)) = compare::at(
        &server.url,
        b,
        "GET",
        compare::Endpoint {
            path: &format!("/grants/{}", v2.0),
            token: t2,
        },
        compare::Endpoint {
            path: &format!("/grants/{}", v3.0),
            token: t3,
        },
        b"",
    )?;
    if a2 != 200 || a3 != 200 {
        return Err(format!("inspect status {a2} {a3}"));
    }
    let mut grant2: Value = grant2;
    grant2["id"] = grant3["id"].clone();
    if grant2 != grant3 {
        return Err(format!("inspection {grant2} vs {grant3}"));
    }
    Ok(())
}
