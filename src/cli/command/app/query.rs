use super::action;
use clap::Command;
pub fn list() -> Command {
    action("list", "List applications", &[], &["url", "token-env"]).visible_alias("ls")
}
pub fn current() -> Command {
    action(
        "current",
        "Show the active release",
        &["name"],
        &["url", "token-env"],
    )
    .visible_alias("inspect")
}
pub fn versions() -> Command {
    action(
        "versions",
        "List releases",
        &["name"],
        &["url", "token-env"],
    )
    .visible_aliases(["releases", "history"])
}
pub fn show() -> Command {
    action(
        "show",
        "Show a release manifest",
        &["name", "release"],
        &["url", "token-env"],
    )
}
pub fn compare() -> Command {
    action(
        "compare",
        "Compare release manifests",
        &["name", "from", "to"],
        &["url", "token-env"],
    )
    .visible_alias("diff")
}

pub fn verify_all() -> Command {
    action(
        "verify-all",
        "Verify all releases",
        &["name"],
        &["url", "token-env"],
    )
}
