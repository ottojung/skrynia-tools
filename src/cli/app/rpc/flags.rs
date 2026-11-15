use super::{Arguments, request::Request};
use std::collections::BTreeMap;
impl Request {
    pub fn into_args(self) -> Arguments {
        let mut flags = BTreeMap::new();
        flags.insert(
            "--apps-dir".into(),
            self.apps_dir.to_string_lossy().into_owned(),
        );
        for (name, value) in [
            ("name", self.name),
            ("release", self.release),
            ("from", self.from),
            ("to", self.to),
            ("format", self.format),
        ] {
            if let Some(value) = value {
                flags.insert(format!("--{name}"), value);
            }
        }
        Arguments::from_rpc(self.action, flags)
    }
}
