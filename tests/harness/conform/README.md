# conform

`conform/` checks the six conformity rules of PLAN.md §2 over a cases tree. It gives one message per violation, in the form `ruleN: <what> at <path>: <how to fix>`. `mod.rs` walks the tree and owns rules 1 and 6, `cases_md.rs` checks rules 2 and 3, `case_dir.rs` checks rules 4 and 5, and `placeholders.rs` finds the `$NAME` tokens a golden may not hold. The self-tests in `tests/runner_selftest.rs` assert every message.
