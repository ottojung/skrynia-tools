mod entry;
use super::{Keys, client};
pub use entry::ensure;
pub fn landing() -> &'static str {
    include_str!("../landing.html")
}
