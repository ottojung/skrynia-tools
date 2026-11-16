use super::tree;
use std::{fs, path::PathBuf};

#[test]
fn npm_vendor_tree_is_excluded_but_owned_source_is_checked() {
    let root = std::env::temp_dir().join(format!("skrynia-vendor-{:032x}", rand_token()));
    let third_party = root.join("node_modules").join("package");
    fs::create_dir_all(&third_party).unwrap();
    for n in 0..12 {
        fs::write(third_party.join(format!("{n}.js")), "").unwrap();
    }
    let source = root.join("owned.rs");
    fs::write(&source, "fn main() {}\n").unwrap();
    let mut issues = Vec::new();
    tree::visit(&root, &mut issues);
    assert!(issues.is_empty(), "{issues:?}");
    fs::write(&source, "fn f() {}\n".repeat(51)).unwrap();
    tree::visit(&root, &mut issues);
    assert_eq!(issues.len(), 1);
    fs::remove_dir_all(root).unwrap();
}
fn rand_token() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
