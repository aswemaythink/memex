//! The log store: how an entry is checked and written (body, front, stream, values, write), how old entries are pruned (prune), and how the entries are read as docs (read), per design §3.4 and §4.3.
//! Cases: none (module map only; each module cites its own cases)

pub mod body;
pub mod front;
pub mod prune;
pub mod read;
pub mod stream;
pub mod values;
pub mod write;
