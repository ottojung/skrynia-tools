//! Process lifecycle for the real full chain: kernel PR125, trusted edge.
mod edge;
mod kernel;
mod p1;
mod port;
pub use edge::EdgeProc;
pub use kernel::{Kernel, op_cap};
pub use p1::transport;
pub use port::free_port;
