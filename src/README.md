# src

`src/` is the crate's source: the `memex` library (`lib.rs`, whose module map is in PLAN.md §4) and the thin `main.rs` binary that turns `memex::run`'s result into the process exit code. Modules land slice by slice, in the order PLAN.md §6 gives.
