pub fn five_apps(cap: &str) -> serde_json::Value {
    serde_json::json!([
        {"name":"along","hosts":["long.a.vau.place"],"origins":["https://long.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"kawunapp","hosts":["kawun.a.vau.place"],"origins":["https://kawun.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"tesicca","hosts":["tesicca.a.vau.place"],"origins":["https://tesicca.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"theorem-lab","hosts":["theorem-lab.a.vau.place"],"origins":["https://theorem-lab.a.vau.place"],"edge_capabilities":[cap]},
        {"name":"sbp","hosts":["sbp.a.vau.place"],"origins":["https://sbp.a.vau.place"],"edge_capabilities":[cap]}
    ])
}

pub fn two_apps(a: &str, b: &str) -> serde_json::Value {
    serde_json::json!([
        {"name":"along","hosts":["long.a.vau.place"],"origins":["https://long.a.vau.place"],"edge_capabilities":[a]},
        {"name":"kawunapp","hosts":["kawun.a.vau.place"],"origins":["https://kawun.a.vau.place"],"edge_capabilities":[b]}
    ])
}
