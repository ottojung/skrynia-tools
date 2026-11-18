use super::*;
#[test]
fn graph_does_not_widen_parent_authority() {
    let mut c = child();
    c.locations = "other/*".into();
    assert_eq!(
        Authority::from_grants([root(), c]).err(),
        Some("widened_scope")
    );
    let mut c = child();
    c.operations.push("delete".into());
    assert_eq!(
        Authority::from_grants([root(), c]).err(),
        Some("widened_operations")
    );
}
#[test]
fn duplicate_tokens_cannot_rebind_to_other_grants() {
    let mut r = root();
    let mut c = child();
    let user = User {
        id: "u".into(),
        hash: "same".into(),
    };
    r.users.push(user.clone());
    c.users.push(user);
    assert_eq!(
        Authority::from_grants([r, c]).err(),
        Some("duplicate_user_token")
    );
}
