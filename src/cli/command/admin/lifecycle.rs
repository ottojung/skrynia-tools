use super::action;
use clap::Command;
pub fn inspect() -> Command {
    action(
        "inspect",
        "Inspect a credential journal",
        &["credentials-file"],
        &[],
    )
}
pub fn resume() -> Command {
    action(
        "resume",
        "Resume grant issuance",
        &["credentials-file"],
        &["url", "accept-duplicate-token-risk"],
    )
}
pub fn revoke_user() -> Command {
    action(
        "revoke-user",
        "Revoke a grant user",
        &["credentials-file", "confirm-grant-id"],
        &["url", "retry-after-uncertain"],
    )
}
pub fn revoke_grant() -> Command {
    action(
        "revoke-grant",
        "Revoke a grant",
        &["credentials-file", "confirm-grant-id"],
        &["url", "retry-after-uncertain"],
    )
}
