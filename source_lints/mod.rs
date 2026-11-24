mod imports;
mod lines;
mod tree;

use std::path::Path;

pub fn check(root: &Path) {
    let mut violations = Vec::new();
    for area in [
        "src",
        "source_lints",
        "tests",
        "examples",
        "benches",
        "v3-trusted-edge-gateway/src",
        "v3-trusted-edge-gateway/tests",
        "v3-trusted-edge-gateway/examples",
        "v3-trusted-edge-gateway/benches",
        "fullchain-integration/src",
        "fullchain-integration/tests",
        "fullchain-integration/examples",
        "fullchain-integration/benches",
        "publish-provider/src",
        "publish-provider/tests",
        "publish-provider/examples",
        "publish-provider/benches",
    ] {
        let folder = root.join(area);
        if folder.is_dir() {
            tree::visit(&folder, &mut violations);
        }
    }
    if let Ok(source) = std::fs::read_to_string(root.join("build.rs")) {
        let count = lines::code_lines(&source);
        if count > 50 {
            violations.push(format!("build.rs: {count} code LOC exceeds 50"));
        }
        imports::check(&root.join("build.rs"), &source, &mut violations);
    }
    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("Rust structural lint: {violation}");
        }
        panic!("{} Rust structural lint violations", violations.len());
    }
}

#[cfg(test)]
mod test_extra;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod vendor_test;
