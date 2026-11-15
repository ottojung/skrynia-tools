//! Native CLI for static applications and optional Skrynia data.
pub(super) use super::{client, deploy, log as protocol_log};
mod admin;
mod app;
mod args;
mod help;
mod log;
mod object;
mod provision;
mod scaffold;
mod secrets;

pub fn run() -> anyhow::Result<()> {
    if matches!(
        std::env::args().nth(1).as_deref(),
        None | Some("help" | "--help")
    ) {
        help::print();
        return Ok(());
    }
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
