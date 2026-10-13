<!-- memex:begin -->
## memex: read intent before code
This repo keeps an intent layer: README.md per folder, a header comment atop each file
(first line = purpose), and CASES.md listing test ids with their claims (ids appear in test names).
- Overview: `memex info`: where everything is (collections, streams, what's new, messages for you).
- Orient: `memex show <path>`: header, cited cases, who cites it, README chain; folders list children.
- Find: `memex search <words>` over READMEs, headers, cases, collections and logs. Use `rg` for symbols.
- Tests for X: `memex show <file>` lists cited case ids; `memex show <ID>` gives claim and evidence.
- Findings: `memex log <stream> [--to <agent>] <<'EOF' … EOF` prints a ref. Hand back the ref, not the report.
- Messages to you: `memex search to:<your-id>`.
- When you change code, update its header/README in the same change and cite the case ids it relies on.
<!-- memex:end -->

## Working on memex

Ground rules (PLAN.md §0):

- Principles (design.md §1): Markdown and a CLI only. No server, daemon, MCP, LLM, vectors or database. Agent-agnostic, minimal surface. A feature earns its place through playtest friction plus a user-approved design.md edit. Logs are scaffolding; knowledge is authored.
- Order of change: design.md, then CASES.md, then code. If design.md is silent or contradicts itself, stop and report. Never invent behaviour.
- Every behaviour is pinned by a conformance case (PLAN.md §2). Unit tests are welcome but do not count as pinning.
- Every file's header: its first sentence is the file's purpose, and it cites the case ids it implements (`Cases: SRCH-FUZZY-*`).
- Every folder has a README.md whose first paragraph says what the folder is.

Brief template (PLAN.md §8). Fill it in for every cook:

```
GOAL    <one sentence>
READ    design.md §<x>; PLAN.md §2 (+ §1 fixes <n>); `memex show <paths>` output (from S2.0)
FILES   <exact list>; tests/cases/<family>/ (only the ids below)
CASES   write CASES.md entries for: <topics/ids>; then fixtures; bless only these ids
RULES   headers cite the ids they implement; touch nothing outside FILES; design silent → stop and report
DONE    DONE(<filter>) + <slice extras>
REPORT  memex log impl --agent cook-<slice> --tag <family> <<'EOF' … EOF → return `report: <ref>` + DONE tail
```

Golden policy (PLAN.md §2, Bless):

- Bless only the case ids your brief names: `MEMEX_BLESS=1 cargo test --test conformance -- <filter>`.
- Review blessed goldens with `git diff tests/cases`; the orchestrator checks `git show --stat`.
- Bless is refused when `CI` is set.
- Goldens may use only the `$SANDBOX` and `$VERSION` redactions: no other placeholder, no wildcard.
