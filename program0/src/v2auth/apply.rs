use super::{MutateErr, MutateKind, MutateReq, cap::Disk, object, paths, state};

pub fn run<D: Disk>(disk: &D, req: MutateReq<'_>) -> Result<bool, MutateErr<D::Err>> {
    paths::ensure(disk).map_err(MutateErr::Disk)?;
    let _guard = super::lock::acquire().map_err(|_| MutateErr::Uncertain)?;
    let auth = state::load(disk).map_err(|e| match e {
        state::StateErr::Disk(e) => MutateErr::Disk(e),
        state::StateErr::Decode => MutateErr::Decode,
        state::StateErr::Uncertain => MutateErr::Uncertain,
        state::StateErr::Missing => MutateErr::MissingAuthority,
        state::StateErr::Exists => MutateErr::AuthorityExists,
    })?;
    let op = match req.kind {
        MutateKind::Create => "create",
        MutateKind::Replace => "modify",
        MutateKind::Delete => "delete",
    };
    let th = paths::hex32(&paths::sha256(&req.token));
    if !auth.users.iter().any(|u| u.hash == th) {
        return Err(MutateErr::InvalidToken);
    }
    if !auth.allows(&th, op, req.key) {
        return Err(MutateErr::Forbidden);
    }
    commit(disk, req)
}

fn commit<D: Disk>(disk: &D, req: MutateReq<'_>) -> Result<bool, MutateErr<D::Err>> {
    let current = object::read(disk, req.key).map_err(MutateErr::Disk)?;
    match req.kind {
        MutateKind::Create if current.is_some() => return Err(MutateErr::Exists),
        MutateKind::Replace | MutateKind::Delete if current.is_none() => {
            return Err(MutateErr::NotFound);
        }
        _ => {}
    }
    if !object::cas_ok(current.as_deref(), req.expected.as_ref()) {
        return Err(MutateErr::ContentMismatch);
    }
    state::mark_uncertain(disk).map_err(MutateErr::Disk)?;
    let result = match req.kind {
        MutateKind::Delete => object::remove(disk, req.key),
        _ => object::write(disk, req.key, req.body),
    };
    if let Err(e) = result {
        return Err(MutateErr::Disk(e));
    }
    state::clear_uncertain(disk).map_err(MutateErr::Disk)?;
    Ok(false)
}
