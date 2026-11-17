use super::action;
use clap::Command;
pub fn command() -> Command {
    action(
        "publish-stable",
        "Publish an app-root service worker or PWA asset",
        &["name", "asset", "file"],
        &["url", "token-env"],
    )
    .mut_arg("asset", |arg| {
        arg.value_parser(["sw.js", "manifest.webmanifest", "icon.svg"])
    })
}
