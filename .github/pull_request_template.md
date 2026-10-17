## Summary

<!-- What changes and why. Link the DESIGN.md section or the issue this closes. -->

## Checklist

- [ ] Order of change: DESIGN.md first (if behaviour changes), then the case in `tests/cases/`, then the code.
- [ ] Every behaviour change is pinned by a conformance case. Unit tests do not count as pinning.
- [ ] Goldens are re-blessed only for the case ids this PR names, and `git diff tests/cases` has been read.
- [ ] Headers and folder READMEs are updated for everything this PR changes.
- [ ] `memex audit <paths>` is clean for the paths this PR changes.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass.
