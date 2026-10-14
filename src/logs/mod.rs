//! The log store: how an entry is checked, written, pruned and read, per design §3.4 and §4.3.
//! Checks: body, front, stream and values. Writes: write. Prunes: prune. Reads docs: read.
//! Cases: none (module map only; each module cites its own cases)

pub mod body;
pub mod front;
pub mod prune;
pub mod read;
pub mod stream;
pub mod values;
pub mod write;
