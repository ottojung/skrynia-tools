//! Source-driven wake cases, split into cohesive <=50-code-LOC modules.
mod authz;
mod crash;
mod fanout;
mod key_write;
mod liveness;
mod neg;
mod quota;
mod restart;
mod stale_ack;
