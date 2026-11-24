use super::{NAMES, hex, manifest, sha256_file, verify_all};
use std::path::Path;

#[test]
fn manifest_is_computed_from_bytes_not_filename() {
    let d = std::env::temp_dir().join(format!("fc-pin-dir-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    for (i, name) in NAMES.iter().enumerate() {
        std::fs::write(d.join(name), format!("bytes-{i}")).unwrap();
    }
    let lines = manifest(&d).unwrap();
    assert_eq!(lines.len(), NAMES.len());
    for name in NAMES {
        let h = hex(&sha256_file(&Path::new(&d).join(name)).unwrap());
        assert!(lines.iter().any(|l| l == &format!("SHA256 {name} {h}")));
    }
    assert!(verify_all(&d).is_err(), "missing manifest must fail closed");
    let pins = NAMES
        .iter()
        .map(|name| format!("{}  {name}\n", hex(&sha256_file(&d.join(name)).unwrap())))
        .collect::<String>();
    let file = d.join("SHA256SUMS");
    std::fs::write(&file, &pins).unwrap();
    assert!(verify_all(&d).is_ok());
    std::fs::write(&file, format!("{pins}{pins}")).unwrap();
    assert!(verify_all(&d).is_err(), "duplicate pins must fail closed");
    std::fs::write(&file, &pins).unwrap();
    std::fs::write(d.join(NAMES[0]), b"tampered").unwrap();
    assert!(verify_all(&d).is_err());
    std::fs::remove_dir_all(&d).unwrap();
}
