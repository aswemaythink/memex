---
memex: cases
---
# Check cases

## SCOPE

- `CHK-SCOPE-01` With no argument, `check` covers the root: its scope line names `.`, counts every walked file and folder, and lists the findings of the whole tree, exit 0.
  design §3.5d Scope and Output. Run: memex check.
- `CHK-SCOPE-02` A file argument checks only that file: the scope line names it with one file and no folder, and the findings of other files are left out, exit 0.
  design §3.5d Scope. Run: memex check src/b.rs.
- `CHK-SCOPE-03` A folder argument checks the files under it and the folder itself, so a folder with no README outside it is not reported, exit 0.
  design §3.5d Scope. Run: memex check src.
- `CHK-SCOPE-04` Several arguments are joined with ` · ` on the scope line, and a file that two of them reach is counted and listed once, exit 0.
  design §3.5d Output. Run: memex check src src/b.rs docs.
- `CHK-SCOPE-05` A path that names nothing is `not found` on stderr with exit 1, and the other paths are still checked.
  design §3.5d Scope and §3.5c. Run: memex check nope.rs src/b.rs.
- `CHK-SCOPE-06` From a subfolder, `.` names the root and `..` names its parent folder, which the scope line labels by its root-relative path.
  design §3.1 step 2 and §3.5d Scope. Run from src/deep: memex check . ; memex check ...
- `CHK-SCOPE-07` A path that leads outside the root names nothing that check covers, so it is `not found` with exit 1 (a ruling: design §3.5d says nothing about other roots).
  design §3.5d Scope. Run: memex check ../outside.rs.
- `CHK-SCOPE-08` A path that names two different entries, one from the cwd and one from the root, is `ambiguous` on stderr with both candidates and exit 1, and no scope is guessed.
  design §3.1 step 2 and §3.5d Scope. Run from src: memex check src.

## GLOB

- `CHK-GLOB-01` A glob argument, one holding `*`, `?` or `[`, is matched against walked paths with the manifest include rules, so `src/*.rs` covers the `.rs` files directly under src and no deeper file, exit 0.
  design §3.5d Scope and §4.2 include. Run: memex check 'src/*.rs'.
- `CHK-GLOB-02` A glob that matches a folder covers that folder and everything under it, exit 0.
  design §3.5d Scope. Run: memex check 'src/*'.
- `CHK-GLOB-03` A glob that matches nothing is not an error: its scope label is printed with zero counts, the output says `nothing to fix`, exit 0.
  design §3.5d Scope. Run: memex check 'docs/*.txt'.

## HEADERS

- `CHK-HEADERS-01` A file with a comment family and no header is listed under `headers` with the header fix line, exit 0.
  design §3.5d kind 1 and §2.1. Run: memex check.
- `CHK-HEADERS-02` An empty family file, zero bytes, is not a finding.
  design §3.5d kind 1 (not empty). Run: memex check.
- `CHK-HEADERS-03` A file with no comment family, such as `notes.txt`, and a Markdown file are not findings, even with no header.
  design §3.5d kind 1 and §2.1. Run: memex check.

## READMES

- `CHK-READMES-01` A walked folder with no README.md is listed under `readmes` by its root-relative path, with the README fix line, exit 0.
  design §3.5d kind 2. Run: memex check.
- `CHK-READMES-02` A root with no README.md is listed as `.`, exit 0.
  design §3.5d kind 2 (the root included). Run: memex check.
- `CHK-READMES-03` A folder that the walk does not enter, here one named by an `.ignore` rule, is not listed, since the report covers walked folders only, exit 0.
  design §3.5d Scope and §4.1 item 2. Run: memex check.

## ONELINERS

- `CHK-ONELINERS-01` A README whose one-liner is over 100 characters is listed by its path with its character count and the one-liners fix line, exit 0.
  design §3.5d kind 4 and §2. Run: memex check.
- `CHK-ONELINERS-02` A one-liner of exactly 100 characters is not a finding.
  design §3.5d kind 4. Run: memex check.
- `CHK-ONELINERS-03` Characters count as Unicode scalar values, so 100 accented letters with a full stop are fine and 101 are listed with `101 characters`.
  design §3.5d kind 4 (Unicode scalar values). Run: memex check.
- `CHK-ONELINERS-04` A header whose one-liner is over 100 characters is listed by its file path, exit 0.
  design §3.5d kind 4 (a header or README). Run: memex check.

## CITES

- `CHK-CITES-01` An explicit reference in a README that resolves to nothing is listed with `→` and the reference as written, with the cites fix line, exit 0.
  design §3.5d kind 6 and §2.3 unresolved. Run: memex check.
- `CHK-CITES-02` An unresolved reference in a header is listed under the file's path, exit 0.
  design §3.5d kind 6. Run: memex check.
- `CHK-CITES-03` An unresolved reference in a collection doc is listed under its `collection:id` label, exit 0.
  design §3.5d kind 6 and §2.2. Run: memex check.
- `CHK-CITES-04` A log's unresolved reference is not a finding, since logs are not checked, exit 0.
  design §3.5d kind 6 (logs are not checked). Run: memex check.
- `CHK-CITES-05` A prefix citation such as `NAME-*` and a bare token that names no doc, such as `UTF-8`, are never findings.
  design §3.5d kind 6 and §2.3 (bare tokens never unresolved). Run: memex check.
- `CHK-CITES-06` A reference to a log entry that the store holds resolves, so it is not a finding.
  design §2.3 (`coll:id` with a known collection) and §3.5d kind 6. Run: memex check.

## IDS

- `CHK-IDS-01` A duplicate id within one collection is listed once as `collection:id`, with the paths that hold it, each once, joined by ` · `, and the ids fix line, exit 0.
  design §3.5d kind 7 and §2.2 Duplicate ids. Run: memex check.
- `CHK-IDS-02` The same id in two different collections is not a duplicate.
  design §2.2 (ids are unique within a collection). Run: memex check.

## LIMIT

- `CHK-LIMIT-01` By default a kind lists 20 findings: with 21 folders lacking a README, the heading reads `showing 20 of 21 (--limit 21 for all)` and 20 lines follow, exit 0.
  design §3.5d Output and §3.3 Limits. Run: memex check.
- `CHK-LIMIT-02` `--limit 1` cuts a kind to its first finding in label order and prints `(--limit M for all)`, exit 0.
  design §3.5d Output. Run: memex check --limit 1.
- `CHK-LIMIT-03` With `--json`, each kind's items are cut by `--limit` while its total counts every finding.
  design §3.6 check. Run: memex check --limit 1 --json.

## JSON

- `CHK-JSON-01` `--json` prints one document with `scope`, `files`, `folders`, `findings`, `kinds` (only the kinds with findings, in §3.5d order) and `errors`, exit 0.
  design §3.6 check. Run: memex check --json.
- `CHK-JSON-02` A path that names nothing is an entry of `errors` with its `arg`, the code `not_found` and its message, stdout holds the document and stderr is empty, exit 1.
  design §3.6 check and §3.5c. Run: memex check nope.rs --json.
- `CHK-JSON-03` An item's `detail` is null where the text has none, and a string where it has one, such as `→ docs/gone.md`.
  design §3.6 check. Run: memex check --json.

## EMPTY

- `CHK-EMPTY-01` A root where every folder has a README, every file has its header and every one-liner fits prints its scope line and then `nothing to fix`, exit 0.
  design §3.5d Output (nothing to fix). Run: memex check.

## EXIT

- `CHK-EXIT-01` Findings exit 0, since a finding is not an error.
  design §3.5d Exit code. Run: memex check.
- `CHK-EXIT-02` A flag that check does not take, such as `--agent`, is a usage error that names check and prints its usage line, exit 2.
  design §3 Flags. Run: memex check --agent cook-1.
- `CHK-EXIT-03` A `--limit` value that is not a whole number is a usage error, exit 2.
  design §3 Flags. Run: memex check --limit x.
- `CHK-EXIT-04` `memex check --help` prints check's help on stdout and exits 0.
  design §3 Flags (`--help` works after any verb). Run: memex check --help.
