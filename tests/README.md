# tests

`tests/` is memex's test suite. The conformance suite in [cases](cases/README.md) is the behavioural spec: every behaviour is pinned there by a case (PLAN.md §0, §2). `conformance.rs` runs it through the shared runner in [harness](harness/README.md), and `runner_selftest.rs` checks the conformity rules against the deliberately broken trees in [runner-selftest](runner-selftest/README.md).
