# memex

memex is a CLI that indexes a repo's authored intent layer (folder READMEs, file headers, CASES.md, collections and logs) so agents read intent before code. Markdown and a CLI only: no server, LLM or database.

## Why

Why memex exists, starting with Vannevar Bush's problem of a record nobody could get back to: [STORY.md](STORY.md).

## Design

The behaviour is specified in [design.md](design.md), which is authoritative.

## Build plan

How memex gets built, slice by slice, is in [PLAN.md](PLAN.md). Behaviour is pinned by the conformance cases in [tests/cases](tests/cases/README.md).

## License

Dual-licensed under MIT OR Apache-2.0, at your option: [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
