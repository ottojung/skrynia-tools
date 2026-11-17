use super::action;
use clap::{ArgGroup, Command};
pub fn new() -> Command {
    action(
        "new",
        "Create an application skeleton",
        &["name", "directory"],
        &[],
    )
}
pub fn deploy() -> Command {
    action(
        "deploy",
        "Publish a new release",
        &["name"],
        &[
            "url",
            "token-env",
            "directory",
            "repo",
            "commit",
            "subdir",
            "output",
        ],
    )
    .group(
        ArgGroup::new("source")
            .args(["directory", "repo"])
            .required(true)
            .multiple(false),
    )
    .mut_arg("repo", |value| value.requires("commit"))
}
pub fn activate() -> Command {
    action(
        "activate",
        "Activate a release",
        &["name", "release"],
        &["url", "token-env"],
    )
    .visible_alias("rollback")
}

pub fn verify() -> Command {
    action(
        "verify",
        "Verify one release",
        &["name", "release"],
        &["url", "token-env"],
    )
}
