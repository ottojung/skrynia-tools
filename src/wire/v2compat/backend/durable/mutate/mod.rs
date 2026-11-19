mod apply;
mod write;
pub(super) use super::{Authority, CompatError, MutationKind, State, data};
use super::{CompatResult, DurableBackend, Mutation, RandomSource, StateStore};

pub fn mutate<S: StateStore, R: RandomSource>(
    backend: &DurableBackend<S, R>,
    req: Mutation<'_>,
) -> CompatResult<bool> {
    apply::run(backend, req)
}
