use super::{Fixture, cases};
use std::fs;
pub fn tampering(fixture: &Fixture, first: &serde_json::Value) {
    let file = fixture
        .apps
        .join("releases/hello")
        .join(first["id"].as_str().unwrap())
        .join("index.html");
    let original = fs::read(&file).unwrap();
    fs::write(&file, b"modified").unwrap();
    let (status, report) = cases::invoke(fixture);
    assert!(!status);
    assert_eq!(report["verified"], 2);
    assert_eq!(report["mismatched"], 1);
    assert_eq!(report["releases"].as_array().unwrap().len(), 3);
    fs::write(&file, &original).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        fs::remove_file(&file).unwrap();
        symlink("/dev/null", &file).unwrap();
        let (status, report) = cases::invoke(fixture);
        assert!(!status);
        assert_eq!(report["errors"], 1);
        assert_eq!(report["verified"], 2);
        fs::remove_file(&file).unwrap();
        fs::write(&file, &original).unwrap();
    }
}
