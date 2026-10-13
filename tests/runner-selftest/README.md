# runner-selftest

`runner-selftest/` holds the deliberately broken cases trees that `runner_selftest.rs` checks the conformity rules against (PLAN.md §2). Each `ruleN-<what>/` tree is a complete cases root with one defect for rule N, and `ok/` is a valid tree that passes every rule. The self-test asserts each tree's exact messages, so a message change has to change the test too.
