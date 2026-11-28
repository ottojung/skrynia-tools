use super::host::Host;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

const VERIFIER: &[u8] = b"operator/security/program0.sha256";
const HEADER: usize = 33;
// Host errnos are authentication denials here, not response codes: -3/-4/-8
// would forge the root's documented UNKNOWN/CLEANUP outcomes before any import.
const INVALID: i32 = -4;

fn exact<H: Host>(host: &mut H, fd: i32, into: &mut [u8]) -> Result<(), i32> {
    let mut at = 0;
    while at < into.len() {
        let n = host.read(fd, &mut into[at..]).map_err(|_| INVALID)?;
        if n == 0 {
            return Err(INVALID);
        }
        at += n;
    }
    Ok(())
}

pub fn require<H: Host>(host: &mut H, input: i32) -> Result<[u8; 32], i32> {
    let mut header = [0u8; HEADER];
    exact(host, input, &mut header)?;
    if header[0] != 1 {
        return Err(INVALID);
    }
    let verifier_fd = host.open(VERIFIER, 1).map_err(|_| INVALID)?;
    let mut expected = [0u8; 32];
    let result = exact(host, verifier_fd, &mut expected).and_then(|()| {
        let mut extra = [0u8; 1];
        if host.read(verifier_fd, &mut extra).map_err(|_| INVALID)? == 0 {
            Ok(())
        } else {
            Err(INVALID)
        }
    });
    let _ = host.close(verifier_fd);
    result?;
    let digest: [u8; 32] = Sha256::digest(&header[1..]).into();
    if bool::from(digest.ct_eq(&expected)) {
        Ok(header[1..].try_into().map_err(|_| INVALID)?)
    } else {
        Err(INVALID)
    }
}
