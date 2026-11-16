use super::Fixture;
use std::fs;

pub fn verify(fixture: &Fixture, first: &serde_json::Value) {
    let root = fixture.apps.to_str().unwrap();
    let cli = |json: bool| {
        let mut args = vec!["app", "audit", "--name", "hello", "--apps-dir", root];
        if json {
            args.extend(["--format", "json"]);
        }
        fixture.cli(&args)
    };
    let clean = cli(true);
    assert!(clean.status.success());
    let good: serde_json::Value = serde_json::from_slice(&clean.stdout).unwrap();
    assert_eq!(good["clean"], true);
    assert_eq!(good["available_releases"], 3);
    let stages = fixture.apps.join("releases/hello");
    let pending = "0123456789ab-0000000000000001";
    let orphan = "0123456789ab-0000000000000002";
    let missing = "0123456789ab-0000000000000003";
    fs::create_dir(stages.join(format!(".{pending}.staging"))).unwrap();
    fs::create_dir(stages.join(orphan)).unwrap();
    let old_id = first["id"].as_str().unwrap();
    let old = fixture
        .apps
        .join("metadata/hello")
        .join(format!("{old_id}.json"));
    let mut record: serde_json::Value = serde_json::from_slice(&fs::read(old).unwrap()).unwrap();
    record["id"] = missing.into();
    record["commit"] = format!("0123456789ab{}", "a".repeat(28)).into();
    fs::write(
        fixture
            .apps
            .join("metadata/hello")
            .join(format!("{missing}.json")),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    let incomplete = cli(true);
    assert!(!incomplete.status.success());
    let report: serde_json::Value = serde_json::from_slice(&incomplete.stdout).unwrap();
    assert_eq!(report["clean"], false);
    assert_eq!(report["abandoned_staging"][0], pending);
    assert_eq!(report["artifact_without_metadata"][0], orphan);
    assert_eq!(report["metadata_without_artifact"][0], missing);
    assert!(
        stages.join(orphan).exists(),
        "audit must never prune releases"
    );
}
