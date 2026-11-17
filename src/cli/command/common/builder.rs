use super::{help, value};
use clap::{Arg, ArgAction, Command};
pub fn action(
    name: &'static str,
    about: &'static str,
    required: &[&'static str],
    optional: &[&'static str],
) -> Command {
    let mut cmd = Command::new(name).about(about);
    for name in required {
        cmd = cmd.arg(arg(name, true));
    }
    for name in optional {
        let required = *name == "url" && cmd.get_name() != "resume";
        cmd = cmd.arg(arg(name, required));
    }
    cmd
}
fn arg(name: &'static str, required: bool) -> Arg {
    let arg = Arg::new(name)
        .long(name)
        .required(required)
        .help(help::text(name));
    if name == "parent-grant-is-root" {
        return arg.action(ArgAction::SetTrue);
    }
    if matches!(
        name,
        "accept-duplicate-token-risk" | "retry-after-uncertain"
    ) {
        return arg
            .num_args(0..=1)
            .default_missing_value("true")
            .value_parser(["true", "false"]);
    }
    let arg = arg.value_name(value::name(name)).num_args(1);
    if name == "url" {
        arg.env("SKRYNIA_URL")
    } else {
        arg
    }
}
