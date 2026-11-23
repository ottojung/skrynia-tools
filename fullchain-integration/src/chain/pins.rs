//! Runtime SHA256 enforcement for the five pinned full-chain artifacts.
//! Every selector verifies bytes on disk before use; a mismatch fails closed.
use sha2::{Digest, Sha256};
use std::path::Path;
use std::{fs, io};

mod table;
#[cfg(test)]
mod tests;

pub use self::table::PINS;

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn sha256_file(p: &Path) -> io::Result<[u8; 32]> {
    Ok(Sha256::digest(fs::read(p)?).into())
}
pub fn verify(p: &Path, expected: &str) -> io::Result<()> {
    let got = hex(&sha256_file(p)?);
    if got == expected {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "SHA256 mismatch {}: expected {expected}, got {got}",
                p.display()
            ),
        ))
    }
}
pub fn verify_all(art: &Path) -> io::Result<()> {
    for (name, pin) in PINS {
        verify(&art.join(name), pin)?;
    }
    Ok(())
}
pub fn manifest(art: &Path) -> io::Result<Vec<String>> {
    PINS.iter()
        .map(|(name, _)| {
            let h = hex(&sha256_file(&art.join(name))?);
            Ok(format!("SHA256 {name} {h}"))
        })
        .collect()
}
