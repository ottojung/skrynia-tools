use super::process;

#[test]
#[ignore = "requires a separately built Skrynia v3 binary"]
fn invalid_root_wasm_reports_useful_startup_error() {
    let bin = std::env::var("SKRYNIA_V3_SERVER_BIN").expect("SKRYNIA_V3_SERVER_BIN");
    let dir = std::env::temp_dir().join(format!(
        "skrynia-tools-bad-wasm-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let wasm_path = dir.join("invalid-root.wasm");
    std::fs::write(&wasm_path, b"not-a-valid-wasm-module").unwrap();
    let err = match process::Server::new(bin, wasm_path.to_string_lossy().into_owned()) {
        Ok(_) => panic!("invalid root wasm must fail startup"),
        Err(e) => e,
    };
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        err.contains("server exited") || err.contains("did not start"),
        "expected startup failure, got: {err}"
    );
    assert!(
        err.contains("--- stderr (bounded) ---"),
        "expected bounded stderr diagnostic, got: {err}"
    );
    let tail = err.split("--- stderr (bounded) ---").nth(1).unwrap_or("");
    assert!(!tail.trim().is_empty(), "expected stderr content, got: {err}");
}
