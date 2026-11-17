mod mutate;
mod query;
mod stable;
use super::common::action;
use clap::Command;
pub fn command() -> Command {
    Command::new("app")
        .about("Manage application releases")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(mutate::new())
        .subcommand(mutate::deploy())
        .subcommand(stable::command())
        .subcommand(mutate::activate())
        .subcommand(query::list())
        .subcommand(query::current())
        .subcommand(query::versions())
        .subcommand(query::show())
        .subcommand(query::compare())
        .subcommand(mutate::verify())
        .subcommand(query::verify_all())
}
