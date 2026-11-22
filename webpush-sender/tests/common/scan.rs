use std::{fs, path::Path};

pub fn scan(dir: &Path, needles: &[&str], found: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            scan(&path, needles, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&path).unwrap();
            for needle in needles {
                if text.contains(needle) {
                    found.push(format!("{}:{needle}", path.display()));
                }
            }
        }
    }
}
