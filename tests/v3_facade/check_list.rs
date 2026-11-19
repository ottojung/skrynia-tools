use super::http::{B, call};

pub fn all_apps(b: &B, t: &str) {
    let r = call(
        b,
        "GET",
        "/store",
        Some("locations=apps/*"),
        t,
        vec![],
        vec![],
    );
    assert_eq!(r.status, 200);
    let v: serde_json::Value = serde_json::from_slice(&r.body).unwrap();
    let keys: Vec<&str> = v["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert!(keys.contains(&"apps/alpha/active.json"));
    assert!(keys.contains(&"apps/beta/active.json"));
    assert!(keys.iter().filter(|k| k.ends_with(".ready")).count() >= 4);
}
