use std::process::Output;
pub fn broken(output: &Output) {
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["clean"], false);
    assert!(
        result["generation_error"]
            .as_str()
            .unwrap()
            .contains("generation")
    );
}
pub fn healthy(output: &Output) {
    let good: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(good["generation_error"].is_null());
    assert!(good["activation_generation"].as_u64().unwrap() > 0);
}
