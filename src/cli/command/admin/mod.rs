mod grants;
mod lifecycle;
use super::common::action;
use clap::Command;
pub fn command() -> Command {
    Command::new("admin")
        .about("Manage grants and credentials")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(grants::grant())
        .subcommand(grants::plan())
        .subcommand(grants::create())
        .subcommand(lifecycle::inspect())
        .subcommand(lifecycle::resume())
        .subcommand(lifecycle::revoke_user())
        .subcommand(lifecycle::revoke_grant())
}
