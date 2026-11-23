//! Bounded-hint decouple proofs: full-queue liveness, cross-namespace scope,
//! and truthful restart/recovery.
mod crossns;
mod fallback_ack;
mod fixture;
mod full_queue;
mod lost_ack;
mod negatives;
mod recovery;
mod sender_setup;
mod support;
