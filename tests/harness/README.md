# harness

`harness/` is the conformance runner's shared code, used by `conformance.rs` and `runner_selftest.rs`. `conform.rs` checks the six conformity rules, `discover.rs` lists families and cases and parses CASES.md, `sandbox.rs` builds each case's sandbox and runs its steps, `transcript.rs` renders and redacts the golden text, and `bless.rs` compares a golden or writes it under `MEMEX_BLESS=1`. The rules and formats it implements are in PLAN.md §2, and the case convention is in [cases](../cases/README.md).
