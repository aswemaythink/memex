//! The verbs that do work: one module per verb that runs.
//! Each takes its request and returns its stdout.
//! Cases: none (module map only; each verb cites its own cases)

pub mod here;
pub mod info;
pub mod init;
pub mod log;
pub mod search;
pub mod show;
