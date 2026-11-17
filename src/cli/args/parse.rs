use super::Arguments;
use std::collections::BTreeMap;
pub fn parse() -> anyhow::Result<Arguments> {
    let matches = super::super::command::root().get_matches();
    let (group, parent) = matches.subcommand().expect("clap requires command");
    let (action, matches) = parent.subcommand().expect("clap requires action");
    let mut flags = BTreeMap::new();
    for name in super::super::command::flag_names() {
        if let Ok(Some(value)) = matches.try_get_one::<String>(name) {
            flags.insert(format!("--{name}"), value.clone());
        } else if let Ok(Some(enabled)) = matches.try_get_one::<bool>(name)
            && *enabled
        {
            flags.insert(format!("--{name}"), "true".into());
        }
    }
    Ok(Arguments {
        group: group.into(),
        action: action.into(),
        flags,
    })
}
