use std::fs;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries: Vec<_> = fs::read_dir(dir).unwrap().map(Result::unwrap).collect();
    assert!(
        entries.len() <= 10,
        "directory entry limit: {} ({})",
        dir.display(),
        entries.len()
    );
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

#[test]
fn fullchain_rust_source_limits() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fullchain-integration/src");
    assert!(root.is_dir(), "fullchain source must exist");
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(!files.is_empty(), "no fullchain Rust source found");
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        let lines = source
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("//")
            })
            .count();
        assert!(
            lines <= 50,
            "source line limit: {} ({lines})",
            path.display()
        );
    }
}
