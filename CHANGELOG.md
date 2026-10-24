# Changelog

The release notes of memex. Each GitHub release takes its notes from the matching section here.

## 0.3.1 - 2026-10-24

- `audit` and `tree` no longer walk another root, whatever the spelling of the path: `memex tree ..` from a repo's root used to read the parent folder, and a path through a symlink that leaves the root (`link/..`, an absolute path to a link) read the tree it points to, taking seconds and printing that tree's warnings. Such a path is now a plain `not found` (outside the root; run memex from there), and the walk, which skips symlinks, names nothing through a link. A symlinked spelling of a path inside the root still works. `show` still reads other roots.
- `info` is fast again on a root with thousands of top-level folders: fitting the `roots` block under the 3,000-byte cap took time quadratic in the number of folders (48 s for 10,000), and now takes one pass (about 1 s, as in 0.2.2). The output is unchanged.

## 0.3.0 - 2026-10-24

- New verb `tree`: the shape of a folder in one call. `memex tree [<path>] [--depth N] [--limit N] [--json]` lists the walked folders and files below the root or a path, two levels down by default, depth first and in the order `show` lists children. Each folder line carries its README's one-liner, or `(no README)` when its subtree holds files a header is expected on, and `(N files · H no header)` counting those files across the whole subtree; each file line carries its header's one-liner, `(no header)`, or a Markdown file's title. The first line gives the scope's totals, and the gap lines `show` prints follow the list.
- `info` prints a `roots` block: the root's subfolders in byte order, each with its README's one-liner (the name alone when it has no README). When the block would pass the 3,000-byte cap it ends with `+N more → memex tree`, and the `to`, `here` and `use` lines are kept. `info --json` gains `roots`, with no cap.
- A README's HTML comment markers are not text: a purpose written as `<!-- … -->` reads as prose, and a paragraph that is empty without them is skipped.
- Text output clips a one-liner on a whole word when that keeps at least 60 of its 120 characters, and drops a trailing `,`, `;` or `:` before the `…`.
- The AGENTS.md snippet that `init` writes now points to `memex tree`; run `memex init` again to refresh it.

## 0.2.2 - 2026-10-22

- Collections work in git worktrees. A manifest root that a linked worktree lacks, such as gitignored generated output that only the main checkout holds, is read from the same path in the main checkout, with one warning per root naming where it was read from: `<collection>: root <root> is not in this worktree; read from the main checkout <path>`. A root the worktree has is still its own, and a root that resolves outside the main checkout is still skipped as leading outside the repo. The main checkout is found from the worktree's layout files, so no git call is made.

## 0.2.1 - 2026-10-20

- `show` no longer prints a README's or Markdown file's frontmatter block as part of its body: a folder's README, a `.md` file shown by path, and each README in the chain now print from after the block, so a README that marks a collection root (`memex: <name>`) reads from its heading. `--json` is unchanged: `text` keeps the whole file.

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
