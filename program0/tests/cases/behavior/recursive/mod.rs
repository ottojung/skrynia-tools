use crate::cases::support::{MemDisk, NoLock};
use skrynia_program0::cap::Eval;
use skrynia_program0::reg::Registry;
use std::sync::Arc;

mod case0;
mod case1;
mod eval;
use eval::cycle_eval::CycleEval;
use eval::self_eval::SelfEval;
