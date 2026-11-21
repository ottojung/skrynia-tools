use super::*;
pub fn list<D: Disk>(disk: &D, dir: &str, iid: &[u8]) -> Result<Vec<[u8; 32]>, StoreErr<D::Err>> {
    let path = store::ready_dir(dir, iid).ok_or(StoreErr::BadId)?;
    let names = disk.list(&path).map_err(StoreErr::Disk)?;
    let mut hashes = Vec::new();
    for name in names {
        let Some(hex) = name.strip_suffix(".ready") else {
            continue;
        };
        let Some(hash) = store_ls::parse_id_name(&format!("{hex}.bin")) else {
            continue;
        };
        let hash: [u8; 32] = hash.try_into().map_err(|_| StoreErr::BadId)?;
        if is_ready(disk, dir, iid, &hash)? {
            hashes.push(hash);
        }
    }
    hashes.sort();
    Ok(hashes)
}
