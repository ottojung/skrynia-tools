use super::host::Host;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

const VERIFIER: &[u8] = b"security/program0.sha256";
const HEADER: usize = 33;

fn exact<H: Host>(host: &mut H, fd: i32, into: &mut [u8]) -> Result<(), i32> {
    let mut at = 0;
    while at < into.len() {
        let n = host.read(fd, &mut into[at..])?;
        if n == 0 {
            return Err(-4);
        }
        at += n;
    }
    Ok(())
}

pub fn require<H: Host>(host: &mut H, input: i32) -> Result<[u8; 32], i32> {
    let mut header = [0u8; HEADER];
    exact(host, input, &mut header)?;
    if header[0] != 1 {
        return Err(-4);
    }
    let verifier_fd = host.open_read(VERIFIER)?;
    let mut expected = [0u8; 32];
    let result = exact(host, verifier_fd, &mut expected).and_then(|()| {
        let mut extra = [0u8; 1];
        if host.read(verifier_fd, &mut extra)? == 0 {
            Ok(())
        } else {
            Err(-4)
        }
    });
    let _ = host.close(verifier_fd);
    result?;
    let digest: [u8; 32] = Sha256::digest(&header[1..]).into();
    if bool::from(digest.ct_eq(&expected)) {
        Ok(header[1..].try_into().map_err(|_| -4)?)
    } else {
        Err(-4)
    }
}
