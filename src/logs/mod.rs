//! The log store: how an entry is written (write) and how old entries are pruned (prune), per design §3.4 and §4.3.
//! Cases: none (module map only; each module cites its own cases)

pub mod prune;
pub mod write;
