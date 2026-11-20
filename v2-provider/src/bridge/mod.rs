mod auth;
mod call;
mod meta;
mod mutate;
mod object;
mod wire;
pub use auth::authority;
pub use call::{Callback, InvokeError};
pub struct InvokeBackend<'a, I: Callback> {
    pub invoke: &'a I,
}
impl<I: Callback> skrynia_tools::wire::v2compat::CompatBackend for InvokeBackend<'_, I> {}
