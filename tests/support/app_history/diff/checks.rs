use serde_json::Value;
pub fn validate(report: &Value, from: &str, to: &str) {
    assert_eq!(report["app"], "hello");
    assert_eq!(report["from"]["id"], from);
    assert_eq!(report["to"]["id"], to);
    assert_eq!(report["from_verified"], true);
    assert_eq!(report["to_verified"], true);
    assert!(report["changes"]["added"].as_array().unwrap().is_empty());
    assert!(report["changes"]["removed"].as_array().unwrap().is_empty());
    let changed = report["changes"]["modified"].as_array().unwrap();
    assert_eq!(changed.len(), 2);
    assert_eq!(changed[0]["path"], "assets/data.txt");
    assert_eq!(changed[1]["path"], "index.html");
    assert_ne!(
        changed[0]["before"]["sha256"],
        changed[0]["after"]["sha256"]
    );
}
