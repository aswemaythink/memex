# Changelog

The release notes of memex. Each GitHub release takes its notes from the matching section here.

## 0.2.0 - 2026-10-20

- Collection roots are found, not listed. A README.md with `memex: <name>` in its frontmatter makes its folder a root of the manifest `<name>`, so a manifest in `.memex/collections/` no longer needs `roots`. An explicit `roots` list still works and is read together with the marked folders, each once. Marked folders honour the same ignores as the rest of the walk (`.gitignore`, `.ignore`, hidden entries, `node_modules`, `target`).
- A README's `memex:` key used to be ignored with a warning. It now marks a root; when it names no manifest, the warning says so and names the manifest file to add.
- `audit` has a new kind, `roots`: a README whose `memex:` names no manifest, a manifest with no root, and a root nested inside another root of the same manifest.
- memex is MIT-licensed, in one LICENSE.md (it was MIT OR Apache-2.0).
- The Claude Code plugin's skill says how a README marks a collection root.

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
