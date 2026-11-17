use super::common::action;
use clap::Command;
pub fn command() -> Command {
    Command::new("object")
        .about("Read and mutate objects")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(action(
            "get",
            "Read an object",
            &["key"],
            &["url", "output", "token-file"],
        ))
        .subcommand(action(
            "list",
            "List scoped object keys",
            &[],
            &["url", "locations", "key", "token-file"],
        ))
        .subcommand(action(
            "create",
            "Create an object",
            &["key", "file"],
            &["url", "token-file"],
        ))
        .subcommand(action(
            "replace",
            "Replace an object",
            &["key", "file"],
            &["url", "token-file", "if-sha256"],
        ))
        .subcommand(action(
            "delete",
            "Delete an object",
            &["key"],
            &["url", "token-file", "if-sha256"],
        ))
}
