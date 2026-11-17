mod entry;
mod stable;
use super::{Keys, client};
pub use entry::ensure;
pub use stable::publish;
pub fn landing() -> &'static str {
    include_str!("../landing.html")
}
