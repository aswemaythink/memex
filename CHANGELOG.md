# Changelog

The release notes of memex. Each GitHub release takes its notes from the matching section here.

## Unreleased

- memex is MIT-licensed, in one LICENSE.md (it was MIT OR Apache-2.0).

## 0.1.1 - 2026-10-17

- The docs site, https://aswemaythink.github.io/memex/, is linked from the README (and so from npm and crates.io) and set as the crate's documentation.
- The Claude Code plugin's homepage is the docs site.

## 0.1.0 - 2026-10-17

The first release.

- Six verbs: `search`, `show`, `log`, `info`, `audit` and `init`; all but `init` take `--json`.
- The intent layer: a README.md per folder and a header atop each file form the graph. Any Markdown file with `memex: <collection>` in its frontmatter joins that collection.
- Citations are walked both ways, so `show` lists what a file cites and what cites it.
- Logs: entries in `.memex/logs/`, kept out of git and capped in size, each addressed by a ref, for handing work between agents.
- `audit` reports missing headers and READMEs, long files, long one-liners, unresolved citations and duplicate ids.
- `init` sets up `.memex/` and the AGENTS.md snippet, and installs the binary when it runs through `npx`.
- A Claude Code plugin: `/plugin marketplace add aswemaythink/memex`.
- Prebuilt binaries for macOS (arm64, x86_64), Linux (static musl, x86_64 and arm64) and Windows (x86_64).
