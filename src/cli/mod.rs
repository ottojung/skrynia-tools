//! Native CLI for static applications and optional Skrynia data.
pub(super) use super::{client, http_app, log as protocol_log};
pub(super) use args::Arguments;
mod admin;
mod app;
mod args;
mod command;
mod log;
mod object;
mod provision;
mod scaffold;
mod secrets;

pub fn run() -> anyhow::Result<()> {
    let args = args::Arguments::parse()?;
    match args.group.as_str() {
        "app" => app::run(&args),
        "object" => object::run(&args),
        "log" => log::run(&args),
        "admin" => admin::run(&args),
        _ => anyhow::bail!(
            "unknown command group: {} (expected app, object, log or admin)",
            args.group
        ),
    }
}
