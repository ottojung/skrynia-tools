mod admin;
mod app;
mod common;
mod log;
mod object;
use clap::Command;
pub use common::flag_names;

pub fn root() -> Command {
    Command::new("skrynia-tools")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Manage applications, objects, logs, and grants in Skrynia")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(app::command())
        .subcommand(object::command())
        .subcommand(log::command())
        .subcommand(admin::command())
}
