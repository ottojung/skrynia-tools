use super::Disk;

pub const INSTALLER_FILE: &str = "operator.cap";

pub enum AuthorityError<E> {
    Disk(E),
    InvalidRecord,
}
pub fn check<D: Disk>(
    disk: &D,
    dir: &str,
    supplied: &[u8],
) -> Result<bool, AuthorityError<D::Err>> {
    if supplied.len() != 32 {
        return Ok(false);
    }
    let path = format!("{}/{}", dir.trim_end_matches('/'), INSTALLER_FILE);
    let Some(stored) = disk.read_all(&path).map_err(AuthorityError::Disk)? else {
        return Ok(false);
    };
    if stored.len() != 32 {
        return Err(AuthorityError::InvalidRecord);
    }
    let mut diff = 0u8;
    for (&a, &b) in supplied.iter().zip(stored.iter()) {
        diff |= a ^ b;
    }
    Ok(diff == 0)
}
