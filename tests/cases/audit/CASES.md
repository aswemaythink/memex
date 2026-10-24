---
memex: cases
---
# Audit cases

## SCOPE

- `AUD-SCOPE-01` With no argument, `audit` covers the root: its scope line names `.`, counts every walked file and folder, and lists the findings of the whole tree, exit 0.
  design §3.5d Scope and Output. Run: memex audit.
- `AUD-SCOPE-02` A file argument audits only that file: the scope line names it with one file and no folder, and the findings of other files are left out, exit 0.
  design §3.5d Scope. Run: memex audit src/b.rs.
- `AUD-SCOPE-03` A folder argument audits the files under it and the folder itself, so a folder with no README outside it is not reported, exit 0.
  design §3.5d Scope. Run: memex audit src.
- `AUD-SCOPE-04` Several arguments are joined with ` · ` on the scope line, and a file that two of them reach is counted and listed once, exit 0.
  design §3.5d Output. Run: memex audit src src/b.rs docs.
- `AUD-SCOPE-05` A path that names nothing is `not found` on stderr with exit 1, and the other paths are still audited.
  design §3.5d Scope and §3.5c. Run: memex audit nope.rs src/b.rs.
- `AUD-SCOPE-06` From a subfolder, `.` names the root and `..` names its parent folder, which the scope line labels by its root-relative path.
  design §3.1 step 2 and §3.5d Scope. Run from src/deep: memex audit . ; memex audit ...
- `AUD-SCOPE-07` A path that leads outside the root names nothing that audit covers, so it is `not found` with exit 1 and the message adds `(outside the root; run memex from there)`.
  design §3.5d Scope. Run: memex audit ../outside.rs.
- `AUD-SCOPE-08` A path that names two different entries, one from the cwd and one from the root, is `ambiguous` on stderr with both candidates and exit 1, and no scope is guessed.
  design §3.1 step 2 and §3.5d Scope. Run from src: memex audit src.
- `AUD-SCOPE-09` A label and a bare id are not scopes: `memex audit cases:<id>` and `memex audit <ID>`, given the fixture's `cases` entry `NAME-001`, each print the `not found` error on stderr (§3.5c text) and exit 1, and in a run with a real path too (the label, then `src/b.rs`), that path is still audited.
  design §3.5d Scope and §3.1 steps 3 and 4. Run: memex audit with the entry's label, then its bare id, then the label and src/b.rs.
- `AUD-SCOPE-10` A unique basename resolves as a scope, as `show` resolves it: `memex audit money.rs` audits the one walked `src/money.rs`, so its `headers` finding is listed and the header-less `src/b.rs` is left out, exit 0.
  design §3.1 step 5 and §3.5d Scope. Run: memex audit money.rs.

## GLOB

- `AUD-GLOB-01` A glob argument, one holding `*`, `?` or `[`, is matched against walked paths with the manifest include rules, so `src/*.rs` covers the `.rs` files directly under src and no deeper file, exit 0.
  design §3.5d Scope and §4.2 include. Run: memex audit 'src/*.rs'.
- `AUD-GLOB-02` A glob that matches a folder covers that folder and everything under it, exit 0.
  design §3.5d Scope. Run: memex audit 'src/*'.
- `AUD-GLOB-03` A glob that matches nothing is not an error: its scope label is printed with zero counts, the output says `nothing to fix`, exit 0.
  design §3.5d Scope. Run: memex audit 'docs/*.txt'.

## HEADERS

- `AUD-HEADERS-01` A file with a comment family and no header is listed under `headers` with the header fix line, exit 0.
  design §3.5d kind 1 and §2.1. Run: memex audit.
- `AUD-HEADERS-02` An empty family file, zero bytes, is not a finding.
  design §3.5d kind 1 (not empty). Run: memex audit.
- `AUD-HEADERS-03` A file with no comment family, such as `notes.txt`, and a Markdown file are not findings, even with no header.
  design §3.5d kind 1 and §2.1. Run: memex audit.

## READMES

- `AUD-READMES-01` A walked folder with no README.md whose subtree holds a file with a comment family is listed under `readmes` by its root-relative path, with the README fix line, exit 0.
  design §3.5d kind 2. Run: memex audit.
- `AUD-READMES-02` A root with no README.md, whose tree holds a file with a comment family, is listed as `.`, exit 0.
  design §3.5d kind 2 (the root included). Run: memex audit.
- `AUD-READMES-03` A folder that the walk does not enter, here one named by an `.ignore` rule, is not listed, since the report covers walked folders only, exit 0.
  design §3.5d Scope and §4.1 item 2. Run: memex audit.
- `AUD-READMES-04` A folder that holds only case data, such as a case folder with case.json and expect.txt and no README, is not a unit, so it is not listed under `readmes`, exit 0.
  design §3.5d kind 2 (a folder of data alone needs none). Run: memex audit.
- `AUD-READMES-05` A folder with no file of its own, only subfolders whose subtree holds a file with a comment family, is listed under `readmes`, and so is the subfolder that holds the code, exit 0.
  design §3.5d kind 2 (the subtree holds a file with a comment family). Run: memex audit.

## LENGTH

- `AUD-LENGTH-01` A file with a comment family whose code lines are 501 is listed under `length` as `501 code lines`, with the length fix line, exit 0.
  design §3.5d kind 3. Run: memex audit.
- `AUD-LENGTH-02` A file of exactly 500 code lines is not a finding, exit 0.
  design §3.5d kind 3 (exceed 500). Run: memex audit.
- `AUD-LENGTH-03` Header and license lines are not code: a file of 520 lines whose header and license blocks hold 29 of them has 491 code lines and is not a finding, exit 0.
  design §3.5d kind 3 and Lines; §2.1 header and license blocks. Run: memex audit.
- `AUD-LENGTH-04` A file with CRLF line ends counts each line once, so 501 code lines in CRLF are listed as `501 code lines`, exit 0.
  design §3.5d Lines (CRLF counts once). Run: memex audit.
- `AUD-LENGTH-05` A last line without a newline counts when it is not empty, so 501 code lines whose last line has no newline are listed as `501 code lines`, exit 0.
  design §3.5d Lines. Run: memex audit.
- `AUD-LENGTH-06` A file longer than the 8 KiB head is counted whole, so 600 code lines in about 16 KiB are listed as `600 code lines`, exit 0.
  design §3.5d Lines (audit reads past the head). Run: memex audit.
- `AUD-LENGTH-07` A file with no comment family, such as a 700-line JSON file, is never counted, and a folder that holds only such data needs no README, exit 0.
  design §3.5d kinds 2 and 3. Run: memex audit.

## ONELINERS

- `AUD-ONELINERS-01` A README whose one-liner is over 100 characters is listed by its path with its character count and the one-liners fix line, exit 0.
  design §3.5d kind 4 and §2. Run: memex audit.
- `AUD-ONELINERS-02` A one-liner of exactly 100 characters is not a finding.
  design §3.5d kind 4. Run: memex audit.
- `AUD-ONELINERS-03` Characters count as Unicode scalar values, so 100 accented letters with a full stop are fine and 101 are listed with `101 characters`.
  design §3.5d kind 4 (Unicode scalar values). Run: memex audit.
- `AUD-ONELINERS-04` A header whose one-liner is over 100 characters is listed by its file path, exit 0.
  design §3.5d kind 4 (a header or README). Run: memex audit.

## THIN

- `AUD-THIN-01` Files of 80 lines or more with a header are listed under `thin`, lowest header share first, each with its `<H>/<T> header lines` detail and no fix line, exit 0.
  design §3.5d kind 5. Run: memex audit.
- `AUD-THIN-02` Equal shares are ordered by path: 4 of 200 lines at `src/a.rs` is listed before 2 of 100 at `src/b.rs`, exit 0.
  design §3.5d kind 5 (ties by path). Run: memex audit.
- `AUD-THIN-03` A file of 79 lines is not listed, and one of 80 lines with a header is, exit 0.
  design §3.5d kind 5 (80 lines or more). Run: memex audit.
- `AUD-THIN-04` A file of 80 lines or more with no header is not listed under `thin`, exit 0.
  design §3.5d kind 5 (that have a header). Run: memex audit.
- `AUD-THIN-05` `--limit 1` lists the first `thin` entry only, and its heading says `(showing 1 of 3; --limit 3 for all; …)`, exit 0.
  design §3.5d Output and §3.3 Limits. Run: memex audit --limit 1.
- `AUD-THIN-06` `thin` is a pointer, not a finding: a root whose only entry is thin prints `0 findings`, then `nothing to fix`, then the `thin` list, exit 0.
  design §3.5d kind 5 (not a finding) and Output (nothing to fix). Run: memex audit.
- `AUD-THIN-07` JSON `findings` leaves thin out, so a root whose only entry is thin has `findings` 0, while `kinds` still lists `thin` with its total and items (a ruling: §3.6 says `kinds` holds the kinds with findings), exit 0.
  design §3.5d kind 5 and §3.6 audit. Run: memex audit --json.

## CITES

- `AUD-CITES-01` An explicit reference in a README that resolves to nothing is listed with `→` and the reference as written, with the cites fix line, exit 0.
  design §3.5d kind 6 and §2.3 unresolved. Run: memex audit.
- `AUD-CITES-02` An unresolved reference in a header is listed under the file's path, exit 0.
  design §3.5d kind 6. Run: memex audit.
- `AUD-CITES-03` An unresolved reference in a collection doc is listed under its `collection:id` label, exit 0.
  design §3.5d kind 6 and §2.2. Run: memex audit.
- `AUD-CITES-04` A log's unresolved reference is not a finding, since logs are not audited, exit 0.
  design §3.5d kind 6 (logs are not audited). Run: memex audit.
- `AUD-CITES-05` A prefix citation such as `NAME-*` and a bare token that names no doc, such as `UTF-8`, are never findings.
  design §3.5d kind 6 and §2.3 (bare tokens never unresolved). Run: memex audit.
- `AUD-CITES-06` A reference to a log entry that the store holds resolves, so it is not a finding.
  design §2.3 (`coll:id` with a known collection) and §3.5d kind 6. Run: memex audit.

## IDS

- `AUD-IDS-01` A duplicate id within one collection is listed once as `collection:id`, with the paths that hold it, each once, joined by ` · `, and the ids fix line, exit 0.
  design §3.5d kind 7 and §2.2 Duplicate ids. Run: memex audit.
- `AUD-IDS-02` The same id in two different collections is not a duplicate.
  design §2.2 (ids are unique within a collection). Run: memex audit.

## LIMIT

- `AUD-LIMIT-01` By default a kind lists 20 findings: with 21 folders lacking a README, the heading reads `(showing 20 of 21; --limit 21 for all; …)` and 20 lines follow, exit 0.
  design §3.5d Output and §3.3 Limits. Run: memex audit.
- `AUD-LIMIT-02` `--limit 1` cuts a kind to its first finding in label order and its heading reads `(showing 1 of M; --limit M for all; <what>)`, exit 0.
  design §3.5d Output. Run: memex audit --limit 1.
- `AUD-LIMIT-03` With `--json`, each kind's items are cut by `--limit` while its total counts every finding.
  design §3.6 audit. Run: memex audit --limit 1 --json.

## JSON

- `AUD-JSON-01` `--json` prints one document with `scope`, `files`, `folders`, `findings`, `kinds` (only the kinds with findings, in §3.5d order) and `errors`, exit 0.
  design §3.6 audit. Run: memex audit --json.
- `AUD-JSON-02` A path that names nothing is an entry of `errors` with its `arg`, the code `not_found` and its message, stdout holds the document and stderr is empty, exit 1.
  design §3.6 audit and §3.5c. Run: memex audit nope.rs --json.
- `AUD-JSON-03` An item's `detail` is null where the text has none, and a string where it has one, such as `→ docs/gone.md`.
  design §3.6 audit. Run: memex audit --json.

## EMPTY

- `AUD-EMPTY-01` A root where every folder has a README, every file has its header, every one-liner fits and no file has 80 lines or more prints its scope line and then `nothing to fix`, exit 0.
  design §3.5d Output (nothing to fix) and kind 5 (thin is not a finding). Run: memex audit.

## EXIT

- `AUD-EXIT-01` Findings exit 0, since a finding is not an error.
  design §3.5d Exit code. Run: memex audit.
- `AUD-EXIT-02` A flag that audit does not take, such as `--agent`, is a usage error that names audit and prints its usage line, exit 2.
  design §3 Flags. Run: memex audit --agent cook-1.
- `AUD-EXIT-03` A `--limit` value that is not a whole number is a usage error, exit 2.
  design §3 Flags. Run: memex audit --limit x.
- `AUD-EXIT-04` `memex audit --help` prints audit's help on stdout and exits 0.
  design §3 Flags (`--help` works after any verb). Run: memex audit --help.

## ROOTS

- `AUD-ROOTS-01` `roots` lists a README whose `memex:` key names no manifest (`→ decisions: no manifest`), a manifest with neither `roots` nor a marking README (`.memex/collections/empty.json  no roots`) and a marked root inside an explicit one (`cases/inner  lib: inside cases`), whose doc is read twice, so `ids` lists it too, exit 0.
  design §3.5d kind 8 and §4.2 Roots. Run: memex audit.
- `AUD-ROOTS-02` A scoped audit lists only the `roots` findings in its scope: the nested root under `cases`, and not the README outside it nor the manifest with no root, which is in scope only when the root is, exit 0.
  design §3.5d kind 8 (scope). Run: memex audit cases --json.

## REFINE

- `AUD-REFINE-01` An unresolved reference in a collection doc's frontmatter field is listed under `cites`, before the unresolved references of its body, exit 0.
  design §3.5d kind 6 and §2.3 (field values are scanned before the body). Run: memex audit.
- `AUD-REFINE-02` Two docs that share an id each give their own unresolved references under the shared label, and a reference that both make is listed once, beside the `ids` finding, exit 0.
  design §3.5d kinds 6 and 7 (a ruling: each doc is read for itself). Run: memex audit.
- `AUD-REFINE-03` A file of exactly 8 KiB, the size of the head, is counted from its head alone: 512 lines with a one-line header are listed as `511 code lines`, exit 0.
  design §3.5d Lines and §2.1 (the 8 KiB head). Run: memex audit.
- `AUD-REFINE-04` A binary file, a `.rs` file with a NUL byte in its head and no header, is skipped: it is no `headers`, `length` or `thin` entry, exit 0.
  design §2.1 (binary files are skipped). Run: memex audit.

## OUTSIDE

Scenario for these: the sandbox holds a README.md with `memex: decisions` (a key that names no manifest, so walking that folder would warn) and `repo/`, the root, with `src/good.rs` (a header), `src/bare.rs` (none) and READMEs.

- `AUD-OUTSIDE-01` `..` from the root is `not found` with `(outside the root; run memex from there)` and nothing else on stderr, since the parent folder is never walked and its README's warning never appears; the scope line stays empty, exit 1, as text and with `--json` (the error in `errors`, code `not_found`).
  design §3.5d Scope (2026-10-24). Run: memex audit .., then memex audit .. --json.
- `AUD-OUTSIDE-02` An absolute path outside the root is the same `not found`, with no walk of that folder, exit 1, as text and with `--json`.
  design §3.5d Scope. Run: memex audit /, then memex audit / --json.
- `AUD-OUTSIDE-03` A path that leaves the root and folds back in (`../repo/src`, `../repo/src/bare.rs`) is judged by where it lands, so it audits that folder or file as the plain path would, exit 0.
  design §3.5d Scope (`.` and `..` folded). Run: memex audit ../repo/src, then memex audit ../repo/src/bare.rs.
- `AUD-OUTSIDE-04` An outside path next to a good one gives its `not found` while the good one is still audited, exit 1: `..` with `src/bare.rs` as text, and `src/good.rs` with `../README.md` with `--json`.
  design §3.5d Scope. Run: memex audit .. src/bare.rs, then memex audit src/good.rs ../README.md --json.
- `AUD-OUTSIDE-05` A path spelled through a symlink that lands inside the root still audits: with `../link` a symlink to `repo`, `../link/src` and `../link/src/bare.rs --json` are judged by their canonical path, so they audit that folder and file as the plain path would, with nothing from the parent folder on stderr, exit 0.
  design §3.5d Scope (a symlinked spelling counts as inside). Setup: a git apply makes the symlink in the sandbox. Run: memex audit ../link/src, then memex audit ../link/src/bare.rs --json.
