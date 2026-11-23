//! Process lifecycle for the real full chain: kernel PR125, trusted edge.
mod edge;
mod edge_cfg;
mod kernel;
mod kernel_fs;
mod p1;
mod pins;
mod port;
mod proc_ready;
pub use edge::EdgeProc;
pub use kernel::{Kernel, op_cap};
pub use p1::transport;
pub use pins::{manifest, verify, verify_all};
pub use port::free_port;
