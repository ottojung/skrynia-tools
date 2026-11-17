use super::common::action;
use clap::Command;
pub fn command() -> Command {
    Command::new("log")
        .about("Work with append-only logs")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(action(
            "init",
            "Initialize a log",
            &["prefix"],
            &["url", "token-file"],
        ))
        .subcommand(action(
            "append",
            "Append an entry",
            &["prefix", "operation-id", "file"],
            &["url", "token-file"],
        ))
        .subcommand(action(
            "read",
            "Read entries",
            &["prefix"],
            &["url", "token-file", "from"],
        ))
}
