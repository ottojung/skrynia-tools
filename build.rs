mod source_lints;

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=source_lints");
    println!("cargo:rerun-if-changed=tests");
    println!("cargo:rerun-if-changed=examples");
    println!("cargo:rerun-if-changed=benches");
    source_lints::check(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
}
