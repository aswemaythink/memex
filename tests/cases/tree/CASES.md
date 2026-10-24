---
memex: cases
---
# Tree cases

## DEFAULT

- `TREE-DEFAULT-01` With no argument `tree` lists the root two levels deep: the first line names `.` with its counts and `(showing N of N)`, the root's entries come at two spaces and their entries at four, folders end in `/` and carry their README's one-liner and counts, and the root's own README.md is not listed, exit 0.
  Scenario: a shop with a README, `build.rs`, a `src` tree three levels deep, `docs`, and data-only `fixtures`. design §3.5e Scope, Depth, What is listed, Lines and First line. Run: memex tree.

## DEPTH

- `TREE-DEPTH-01` `--depth 1` lists the root's own entries only, and each folder still carries its whole subtree's counts, exit 0.
  design §3.5e Depth. Run: memex tree --depth 1.
- `TREE-DEPTH-02` `--depth 3` lists one level more than the default, depth first, exit 0.
  design §3.5e Depth. Run: memex tree --depth 3.

## SCOPE

- `TREE-SCOPE-01` A folder argument lists that folder: the first line names it and gives its counts, and its entries sit at two spaces with their root-relative labels, exit 0.
  design §3.5e Scope and Lines. Run: memex tree src, then memex tree src/deep --depth 1.
- `TREE-SCOPE-02` A file argument prints the first line and that file's line alone, with the file's own counts: a file with a header reads `1 file`, one without reads `1 file · 1 no header` and `(no header)` with its gap line, a README.md and a file with no comment family are listed by the same rules (a title, or the label alone) and count 0 files, exit 0.
  design §3.5e Scope and Lines. Run: memex tree src/main.rs, src/util.rs, src/README.md, docs/data.json.
- `TREE-SCOPE-03` A path that names nothing is a fatal `not found` with exit 1, on stderr as text and as the error document on stdout with `--json`.
  design §3.5e Scope, §3.5c and §3.6 Fatal errors. Run: memex tree nope, then memex tree nope --json.
- `TREE-SCOPE-04` A glob, a label and a bare id are not paths, so each is `not found` with exit 1.
  design §3.5e Scope. Scenario: the shop plus a CASES.md that declares `SHOP-001`. Run: memex tree 'src/*', then memex tree with that entry's label, then with its bare id.
- `TREE-SCOPE-05` With no argument `tree` lists the root even from a subfolder, a relative path resolves against the cwd as `show` resolves one, and `..` names the parent folder, exit 0.
  design §3.5e Scope and §3.1 step 2. Run from src: memex tree, memex tree deep --depth 1; from src/deep: memex tree ..
- `TREE-SCOPE-06` A path that names two different entries, one from the cwd and one from the root, is a fatal `ambiguous` on stderr with both candidates and exit 1.
  design §3.1 step 2 and §3.5e Scope. Run from src: memex tree src.

## USAGE

- `TREE-USAGE-01` `--depth 0` is a usage error with exit 2 that names the value and the grammar line, and with `--json` the error is the usage envelope.
  design §3.5e Depth and §3.6. Run: memex tree --depth 0, then with --json.
- `TREE-USAGE-02` `--depth x` is a usage error, and `--depth` with no value is one too, exit 2.
  design §3.5e Depth. Run: memex tree --depth x, then memex tree --depth.
- `TREE-USAGE-03` Two paths are a usage error, `unexpected argument`, exit 2.
  design §3.5e Scope. Run: memex tree src docs.
- `TREE-USAGE-04` An empty path is a usage error, as it is for `audit`, exit 2 (a ruling: design §3.5e says nothing about it).
  design §3.5e Scope and §3.5d. Run: memex tree "".

## HELP

- `TREE-HELP-01` `memex tree --help` prints the grammar line and what each flag does, and `memex --help` lists `tree` in its grammar block and its verb list after `info`, exit 0.
  design §3 and §3.5e. Run: memex tree --help, then memex --help.

## NOHDR

- `TREE-NOHDR-01` A folder with no README whose subtree holds files a header is expected on reads `(no README)`, then its counts, and the gap lines follow the list, `(no README)` first, once each, exit 0.
  Scenario: `raw/` has two files and one lacks a header; `lib/` has a README-less single file with a header. design §3.5e Lines and Gaps. Run: memex tree.
- `TREE-NOHDR-02` A folder of data alone gets no placeholder and no counts, whether or not it has a README, and there is no gap line, exit 0.
  Scenario: `data/` (JSON and text, with a subfolder) has no README, and `docs/` has a README and a JSON file. design §3.5e Lines (a unit folder, as audit's readmes judges it). Run: memex tree.

## FILE

- `TREE-FILE-01` A file's line is its header's one-liner cut at 120 characters on a whole word, `(no header)` for a file a header is expected on that has none, a Markdown file's title, or the label alone for an empty Markdown file, a file with no comment family, an empty file and a binary file, and those last three are not counted, exit 0.
  design §3.5e Lines and Counts, §3.3 and §2 One-liners. Run: memex tree.

## COUNT

- `TREE-COUNT-01` A folder's counts read `(N files · H no header)`; one file is `1 file`, and ` · H no header` is left out when H is 0, exit 0.
  Scenario: folders with one file, two files, three files with one lacking a header, and one file with none; the first line gives the root's totals. design §3.5e Counts, Lines and First line. Run: memex tree.
- `TREE-COUNT-02` The counts of a folder cover its whole subtree whatever the depth, and leave out empty files, binary files and files with no comment family, exit 0.
  design §3.5e Depth and Counts. Run: memex tree --depth 1, then memex tree a --depth 1.

## GAP

- `TREE-GAP-01` The gap line of a placeholder is printed once after the list however many lines use it, and a listing with no placeholder prints none, exit 0.
  Scenario: two files without a header under `mixed/`; `tree clean` has none. design §3.5e Gaps. Run: memex tree, then memex tree clean.

## LIMIT

- `TREE-LIMIT-01` `--limit` bounds the entries printed, cutting the depth-first list at its end, and the first line says `(showing N of M; --limit M for all)` only when the list was cut; `--limit 0` prints the first line alone, exit 0.
  design §3.5e Limit and First line. Run: memex tree --limit 4, --limit 0, --limit 9.

## ORDER

- `TREE-ORDER-01` Entries come depth first, a folder's entries straight after it, each folder's entries in the order `show` lists children: by name in byte order, so `B` before `a`, and the folder `a` before `a-x.rs` and `a.rs`, exit 0.
  design §3.5e What is listed and §3.3. Run: memex tree.

## JSON

- `TREE-JSON-01` `--json` prints `{scope, depth, files, no_header, total, items}`; each item has `path` (no trailing `/`), `kind`, `depth`, the full `title`, `gap` (`"readme"`, `"header"` or `null`), and a folder also `files` and `no_header`, exit 0.
  design §3.6 `tree`. Run: memex tree --json.
- `TREE-JSON-02` With `--limit 1` `items` holds the printed entry and `total` the number within the depth, and a file path gives its one item, exit 0.
  design §3.6 `tree`. Run: memex tree --json --limit 1, then memex tree b/y.rs --json.

## SKIP

- `TREE-SKIP-01` What the walk skips is absent from the listing and the counts: a hidden folder, `node_modules`, `target` and a folder named in `.ignore`, exit 0.
  design §3.5e What is listed and §4.1 item 2. Run: memex tree.

## OUTSIDE

Scenario for these: the sandbox holds a README.md with `memex: decisions` (a key that names no manifest, so walking that folder would warn) and `repo/`, the root, with `src/good.rs` (a header), `src/bare.rs` (none) and READMEs.

- `TREE-OUTSIDE-01` `..` from the root is a fatal `not found` with `(outside the root; run memex from there)` and nothing else on stderr, since the parent folder is never walked and its README's warning never appears, exit 1, as text and with `--json` (the error document on stdout).
  design §3.5e Scope and §3.5d Scope (2026-10-24). Run: memex tree .., then memex tree .. --json.
- `TREE-OUTSIDE-02` An absolute path outside the root is the same fatal `not found`, with no walk of that folder, exit 1, as text and with `--json`.
  design §3.5e Scope. Run: memex tree /, then memex tree / --json.
- `TREE-OUTSIDE-03` A path that leaves the root and folds back in (`../repo/src`, `../repo/src/bare.rs`) is judged by where it lands, so it lists that folder or file as the plain path would, exit 0.
  design §3.5e Scope and §3.5d Scope (`.` and `..` folded). Run: memex tree ../repo/src, then memex tree ../repo/src/bare.rs.

- `TREE-OUTSIDE-04` A path spelled through a symlink that lands inside the root still lists: with `../link` a symlink to `repo`, `../link/src` and `../link/src/bare.rs --json` are judged by their canonical path, so they list as the plain path would, exit 0.
  design §3.5e Scope and §3.5d Scope (a symlinked spelling counts as inside). Setup: a git apply makes the symlink in the sandbox. Run: memex tree ../link/src, then memex tree ../link/src/bare.rs --json.

## READ

- `TREE-READ-01` A folder whose README.md cannot be read reads `(no README)` on its line when its subtree holds files a header is expected on, as a README that is missing does, and the one unreadable-file warning is printed once on stderr, exit 0.
  design §3.5e Rulings (an unreadable README counts as none) and §4.2. Run: memex tree, with lib/README.md at mode 000.

## SYMLINK

- `TREE-SYMLINK-01` A path through a symlink inside the root names nothing, because the walk skips links, and no other tree is read: with `repo/out` a link to `../other` (whose README carries a `memex:` key that names no manifest, so any walk of it would warn), `tree ../repo/out` and `audit ../repo/out` are `not found` with only that error on stderr and exit 1, while `tree ../repo/out/..` folds to the root and lists it, exit 0.
  design §3.5d Scope and §3.5e Scope (2026-10-24). Setup: a git apply makes the link. Run: memex tree ../repo/out, then memex audit ../repo/out, then memex tree ../repo/out/...
