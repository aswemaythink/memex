<!-- memex:begin -->
## memex: read intent before code
This repo keeps an intent layer. The graph: README.md per folder and a header comment atop each file
(first line = purpose). Dimensions: any Markdown file with `memex: <collection>` in its frontmatter
joins that collection, such as CASES.md (`memex: cases`): test ids with their claims; each id is in its test's name or in a comment on it.
- Overview: `memex info`: where everything is (graph, collections, streams, messages for you).
- Orient: `memex show <path>`: header, cites, who cites it, README chain; folders list children. `memex tree [path]`: folders and files two levels down, with one-liners and header coverage.
- Find: `memex search <words>` over READMEs, headers, collections and logs. Use `rg` for symbols.
- Tests for X: `memex search <words> --in cases` (or the collection `memex info` names); `memex show <ID>` gives claim and evidence; `rg <ID>` finds the test.
- Findings: `memex log <stream> [--to <agent>] <<'EOF' … EOF` prints a ref. Hand back the ref, not the report.
- Messages to you: `memex search to:<your-id>`.
- When you change code, update its header/README in the same change, then run `memex audit <paths>` and fix what it lists. Cite ids and paths; don't copy.
<!-- memex:end -->

## Working on memex

Ground rules:

- Principles (DESIGN.md §1): Markdown and a CLI only. No server, daemon, MCP, LLM, vectors or database. Agent-agnostic, minimal surface. A feature earns its place through playtest friction plus a user-approved DESIGN.md edit. Logs are scaffolding; knowledge is authored.
- Order of change: DESIGN.md, then CASES.md, then code. If DESIGN.md is silent or contradicts itself, stop and report. Never invent behaviour.
- Every behaviour is pinned by a conformance case. Unit tests are welcome but do not count as pinning.
- Every file's header: its first sentence is the file's purpose; citing case ids is optional.
- Every folder has a README.md whose first paragraph says what the folder is.

Brief template. Fill it in for every cook:

```
GOAL    <one sentence>
READ    DESIGN.md §<x>; `memex show <paths>` output
FILES   <exact list>; tests/cases/<family>/ (only the ids below)
CASES   write CASES.md entries for: <topics/ids>; then fixtures; bless only these ids
RULES   headers state the file's purpose; touch nothing outside FILES; design silent → stop and report
DONE    DONE(<filter>) + <slice extras>
REPORT  memex log impl --agent cook-<slice> --tag <family> <<'EOF' … EOF → return `report: <ref>` + DONE tail
```

Golden policy:

- Bless only the case ids your brief names: `MEMEX_BLESS=1 cargo test --test conformance -- <filter>`.
- Review blessed goldens with `git diff tests/cases`; the orchestrator checks `git show --stat`.
- Bless is refused when `CI` is set.
- Goldens may use only the `$SANDBOX` and `$VERSION` redactions: no other placeholder, no wildcard.

### Code quality bar

- At most 365 lines per file.
- Each file opens with a `//!` header: its first sentence is the file's purpose; citing case ids is optional.
- Each folder has a README.md whose first paragraph says what the folder is.
- Functions have at most 60 lines and cognitive complexity 15, and take at most 6 arguments.
- No `#[allow]` or `#[expect]` in src/: fix the code instead. tests/ may use `#[allow(dead_code)]` only.
- Portable by construction: paths in labels and JSON go through one `/` normaliser, special names are matched against directory listings (never `exists()`), `cfg` code has both arms, and `cargo clippy --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` is part of every done check.
- CI enforces all of it: clippy with `-D warnings` (lints in Cargo.toml, thresholds in clippy.toml) and `tests/quality.rs`.

### Parallel slices (lessons from the 2026-10-14 build)

- Shared files belong to the orchestrator, not to parallel slices: `tests/cases/README.md` (the hub), `src/README.md`, `src/lib.rs` dispatch lines, and any family README or CASES.md that another running slice also touches. A brief names which of these, if any, the slice may edit.
- A slice that adds cases to a family another slice is also extending adds them under its own new `## TOPIC` heading, never inside an existing topic.
- Read-path slices (corpus, walk, index, cite, resolve, any verb) carry a timing budget in DONE: release `memex info` and `memex search <word>` in /Users/ryn/Developer/reference-ui (read-only) under 90 ms and 115 ms (and `memex show packages` under 95 ms, `memex audit` under 150 ms; on the 100k stress tree `audit` under 2 s), the budgets tests/stress/budget.rs enforces; on a loaded machine, judge by an interleaved A/B against main's binary instead.
- Rulings made where DESIGN.md was silent are committed to DESIGN.md and listed in the PR description, so the user can review them in one place.
