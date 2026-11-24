//! Allocate directory ids and encode a split directory tail.
use super::{DIR_CAP, Disk, StoreErr, format, serialize, state};

#[allow(clippy::type_complexity)]
pub fn dir<D: Disk>(
    st: &mut state::State,
    mut nd: u32,
) -> Result<(u32, Option<(u32, Vec<u8>)>), StoreErr<D::Err>> {
    let inc = |n: u32| n.checked_add(1).ok_or(StoreErr::BadRecord);
    if st.di < st.root.len() {
        st.root.remove(st.di);
    }
    let tail = if st.dir.len() > DIR_CAP {
        let t = st.dir.split_off(st.dir.len() / 2);
        nd = inc(nd)?;
        serialize::put_dir(&mut st.root, t[0].0, nd);
        Some((nd, format::seal(serialize::dir(&t))))
    } else {
        None
    };
    if !st.dir.is_empty() {
        nd = inc(nd)?;
        serialize::put_dir(&mut st.root, st.dir[0].0, nd);
    }
    if st.root.len() > super::super::ROOT_CAP {
        return Err(StoreErr::BadRecord);
    }
    Ok((nd, tail))
}
