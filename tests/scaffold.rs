use std::{fs, process::Command};

#[test]
fn scaffold_is_standalone_static_app() {
    let root =
        std::env::temp_dir().join(format!("skrynia-scaffold-{:032x}", rand::random::<u128>()));
    let status = Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
        .args([
            "app",
            "new",
            "--name",
            "hello",
            "--directory",
            root.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(root.join("public/index.html").is_file());
    assert!(root.join("Makefile").is_file());
    assert!(!root.join("store").exists());
    let build = Command::new("make")
        .arg("build")
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(build.success());
    assert_eq!(
        fs::read(root.join("public/index.html")).unwrap(),
        fs::read(root.join("build/index.html")).unwrap()
    );
    fs::remove_dir_all(root).unwrap();
}
