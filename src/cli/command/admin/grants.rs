use super::action;
use clap::Command;
pub fn grant() -> Command {
    action(
        "grant",
        "Create a grant or issue another user",
        &[],
        &[
            "url",
            "parent-grant",
            "parent-grant-is-root",
            "existing-grant",
            "locations",
            "operations",
            "max-bytes",
            "max-objects",
            "credentials-out",
        ],
    )
    .mut_arg("parent-grant", |value| {
        value.conflicts_with_all(["parent-grant-is-root", "existing-grant"])
    })
    .mut_arg("parent-grant-is-root", |value| {
        value.conflicts_with("existing-grant")
    })
}
pub fn plan() -> Command {
    action(
        "plan",
        "Preview an application grant",
        &["name", "max-bytes", "max-objects"],
        &["id"],
    )
}
pub fn create() -> Command {
    action(
        "create",
        "Create an application grant",
        &[
            "name",
            "max-bytes",
            "max-objects",
            "credentials-out",
            "parent-token-file",
        ],
        &["url", "id"],
    )
}
