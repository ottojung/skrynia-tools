//! Allocate leaf ids and encode a split leaf tail.
use super::{Disk, LEAF_CAP, StoreErr, format, serialize, state};

#[allow(clippy::type_complexity)]
pub fn leaf<D: Disk>(
    st: &mut state::State,
    mut nl: u32,
) -> Result<(u32, Option<(u32, Vec<u8>)>), StoreErr<D::Err>> {
    let inc = |n: u32| n.checked_add(1).ok_or(StoreErr::BadRecord);
    let mut bytes = std::mem::take(&mut st.leaf);
    let tail = if bytes.len() > LEAF_CAP {
        let t = bytes.split_off(bytes.len() / 2);
        nl = inc(nl)?;
        serialize::put_dir(&mut st.dir, t[0].0, nl);
        Some((nl, format::seal(serialize::leaf(&t))))
    } else {
        None
    };
    if !bytes.is_empty() {
        nl = inc(nl)?;
        serialize::put_dir(&mut st.dir, bytes[0].0, nl);
    }
    st.leaf = bytes;
    Ok((nl, tail))
}
