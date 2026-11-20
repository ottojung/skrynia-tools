mod call;
mod meta;
mod object;
pub use call::{Callback, InvokeError};
pub struct InvokeBackend<'a, I: Callback> {
    pub invoke: &'a I,
}
impl<I: Callback> skrynia_tools::wire::v2compat::CompatBackend for InvokeBackend<'_, I> {}
