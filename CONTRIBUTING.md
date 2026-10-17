# Contributing to memex

memex is a small Rust CLI. Its working rules are in [AGENTS.md](AGENTS.md), and the behaviour it must have is in [DESIGN.md](DESIGN.md). This file covers how to build, test and land a change.

## Order of change

1. **DESIGN.md.** A change in behaviour starts as an edit to [DESIGN.md](DESIGN.md), approved by a maintainer. If DESIGN.md is silent or contradicts itself, open an issue rather than guessing.
2. **Conformance case.** Pin the behaviour with a case in [tests/cases](tests/cases/README.md): an entry in the family's `CASES.md`, and a case folder with its `case.json`. Unit tests are welcome, but they do not count as pinning.
3. **Code.** Change the code, and update its header and its folder's README in the same change.

## Build and test

```sh
cargo build --locked
cargo test --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
```

`cargo test --locked` runs the conformance suite, the quality gate in `tests/quality.rs` and the self-host checks in `tests/selfhost.rs`. Tests must leave the tree clean: CI fails if they leave changes in the working tree. The opt-in stress suite and its timing budgets run with `cargo test --release --test stress -- --ignored`; see [tests/stress](tests/stress/README.md).

### Goldens

Each case has a golden transcript, `expect.txt`, which bless writes. Bless only the case ids your change touches:

```sh
MEMEX_BLESS=1 cargo test --test conformance -- <filter>
```

Read the result with `git diff tests/cases` before you commit. Bless is refused when `CI` is set, and CI fails if `MEMEX_BLESS` is set.

## Code quality

The full bar is in [AGENTS.md](AGENTS.md). In short:

- At most 365 lines per file.
- Each file opens with a `//!` header whose first sentence states its purpose.
- Each folder has a `README.md` whose first paragraph says what the folder is.
- Functions have at most 60 lines, cognitive complexity 15 and six arguments.
- No `#[allow]` or `#[expect]` in `src/`. Fix the code instead.
- Portable by construction: one `/` path normaliser, special names matched against directory listings (never `exists()`), and `cfg` code with both arms.

Run `cargo run -q -- audit <paths>` (or `memex audit <paths>` with an installed binary) on the paths you change, and fix what it lists.

## Pull requests

1. Branch from `main`. `main` is protected: changes land only through a pull request.
2. Fill in the pull request template and tick its checklist.
3. CI must pass on Ubuntu, macOS and Windows, and on the MSRV build (Rust 1.88).
4. A maintainer squash-merges the pull request.

## Releases

A maintainer releases by pushing a tag `vX.Y.Z` on `main`. cargo-dist then builds the binaries and installers (see [dist-workspace.toml](dist-workspace.toml) and [release.yml](.github/workflows/release.yml)). Each version has its own section in [CHANGELOG.md](CHANGELOG.md), and the GitHub release takes its notes from that section.
