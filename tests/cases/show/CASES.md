# Show cases

## FILE

- `SHOW-FILE-01` `show` of a `.rs` file prints its header under its `===` line, then `cites` and `cited by` (both none), then the chain of its folder's README and the root README, exit 0.
  design §3.3 File and Chain. Run: memex show src/cli.rs.
- `SHOW-FILE-02` A file with no header, such as `notes.txt`, shows `(no header)` as its body and the root README as its only chain entry, exit 0.
  design §3.3 File and §2.1 Header detection. Run: memex show notes.txt.
- `SHOW-FILE-03` A relative path is resolved against the cwd, so `../cli.rs` run from `src/deep` shows the section labelled `src/cli.rs`, exit 0.
  design §3.1 Resolution order, step 2. Run: memex show ../cli.rs with cwd repo/src/deep.

## MD

- `SHOW-MD-01` A Markdown file that is not a README shows its whole text as its body, not a header, and its chain starts at its own folder, exit 0.
  design §3.3 File: the full content for .md. Run: memex show plain/notes.md.
- `SHOW-MD-02` A README.md file shows its whole text, and its chain starts one folder above its own, so it lists only the root README, exit 0.
  design §3.3 Chain: for a README.md the chain starts at the folder above its own. Run: memex show src/README.md.

## DIR

- `SHOW-DIR-01` A folder with a README shows the README as its body, then its children without that README.md, each subfolder with its README one-liner or `(no README)` and each file with its header one-liner or `(no header)`, then cites, cited by and the chain, exit 0.
  design §3.3 Folder: the folder's own README.md is left out of its children. Run: memex show src.
- `SHOW-DIR-02` A folder without a README shows `(no README)` as its body and still lists its children, which is not an error, exit 0.
  design §3.3 Folder: a folder without a README lists its children and is not an error. Run: memex show plain.

## ROOT

- `SHOW-ROOT-01` `show .` shows the root README as its body, then the root's children other than that README, and its chain is `(none)` because nothing lies above the root, exit 0.
  design §3.3: show . is the map of the root; design §3.1 and PLAN §6 S2.0: `.` is the root. Run: memex show . in the root.
- `SHOW-ROOT-02` Without a `.memex/` folder, show still reads the intent layer from the cwd and prints the same root section, exit 0.
  design §4.1 item 1 and §3.5b: reads work without init. Run: memex show . in a repo with no .memex/.
- `SHOW-ROOT-03` `show .` run from the subfolder `repo/sub` prints the root map under its `=== . ===` line, not the subfolder's, exit 0.
  design §3.1 Resolution order, step 2: `.` is the root's own label, so it always means the root. Run: memex show . with cwd repo/sub.

## LOG

- `SHOW-LOG-01` A log ref shows its body under its `===` line, then a `fields` block with the frontmatter in the order the file gives it (lists as `[a, b]`), then cites and cited by, and no chain, exit 0.
  design §3.3 Log or collection doc, and §4.3 frontmatter. Run: memex show impl:20261013-120000-show-walks-hub-readmes.
- `SHOW-LOG-02` A log ref is not found when the repo has no `.memex/` store, so it exits 1 with `memex: not found:` on stderr and nothing on stdout.
  design §4.1 item 3: the store is read from `.memex/logs/`; design §3.1 step 3. Run: memex show with the log ref in a repo with no .memex/.

## UP

- `SHOW-UP-01` `--up 0` prints the file section alone, so no `chain` line appears, exit 0.
  design §3.3 File: `--up 0` prints the file alone. Run: memex show src/cli.rs --up 0.
- `SHOW-UP-02` `--up 1` limits the chain to one README, so a file in a folder with no README gets only the nearest ancestor's README, exit 0.
  design §3.3 Chain and `--up N`. Run: memex show src/deep/long.rs --up 1.

## LIMIT

- `SHOW-LIMIT-01` `--limit 1` lists one child, `cli.rs`, and prints `showing 1 of 3 (--limit 3 for all)`, exit 0.
  design §3.3 Limits: the children list uses --limit and always prints the limits line. Run: memex show src --limit 1.
- `SHOW-LIMIT-02` `--limit 0` lists no children and still prints `showing 0 of 3 (--limit 3 for all)`, with no `(none)` line because the folder is not empty, exit 0.
  design §3.3 Limits: the limits line always prints. Run: memex show src --limit 0.
- `SHOW-LIMIT-03` A folder with 51 children lists the first 50 and prints `showing 50 of 51 (--limit 51 for all)`, exit 0.
  design §3.3 Limits: the children list uses --limit, default 50, and adds the hint only when it is cut. Run: memex show many.

## SEP

- `SHOW-SEP-01` Two refs print two sections, each under its own `===` line, with one blank line between them, exit 0.
  design §3.3 (a section per ref, separated by `===` lines) and PLAN §6 S2.0 layout. Run: memex show src/cli.rs notes.txt.

## ERR

- `SHOW-ERR-01` A ref that is not found prints `memex: not found: <ref>` on stderr while the other refs still print, and the exit code is 1.
  design §3.3 Exit code: the others still print and the exit code is 1. Run: memex show nope.rs src/cli.rs.
- `SHOW-ERR-02` A relative path that the cwd and the root both hold, and that differ, is ambiguous: stdout is empty, stderr names the ref and lists both candidates, and the exit code is 1.
  design §3.1 Resolution order, step 2: if both exist and differ, the result is ambiguous. Run: memex show README.md with cwd repo/src.
- `SHOW-ERR-03` An empty ref is a usage error: `show ''` prints `memex: empty ref` and the show usage line on stderr, prints nothing on stdout and exits 2, and with `--json` it prints the usage envelope, exit 2.
  design §3.3 (refs are paths or refs) and §3.6 exit codes: 2 is a usage error. Run: memex show '' and memex show '' --json.

## IO

- `SHOW-IO-01` A requested file that cannot be read is an io error for its own ref, `memex: cannot read <arg> (PermissionDenied)`, while the other refs still print and the exit code is 1; with `--json` the same error is in `errors` with code `io`, and the exit code is 1.
  design §3.3 (a requested unreadable file is that ref's io error, so show exits 1), §4.2 (the same file met while loading the corpus is also one warning on stderr) and §3.6 exit codes. Run: memex show src/cli.rs src/locked.rs notes.txt, then the same with --json, where src/locked.rs has mode 000.
- `SHOW-IO-02` A requested folder that cannot be listed is an io error for its own ref, `memex: cannot read <arg> (PermissionDenied)`, while the other refs still print and the exit code is 1; with `--json` the same error is in `errors` with code `io`, and the exit code is 1.
  design §3.3 (a requested folder that can't be listed is that ref's io error, so show exits 1), §4.2 (the same folder met while loading the corpus is also one warning on stderr) and §3.6 exit codes. Run: memex show src/cli.rs src/locked notes.txt, then the same with --json, where src/locked has mode 000.

## DUP

- `SHOW-DUP-01` A ref given twice prints its section once, at the position of its first use, so `show src/cli.rs notes.txt src/cli.rs` prints two sections in that order, exit 0.
  design §3.3 gives a section for each ref in the order given; that a repeat prints once is the S5.2 ruling, which §3.3 should state. Run: memex show src/cli.rs notes.txt src/cli.rs.
- `SHOW-DUP-02` A ref that is not found and is given twice is reported once on stderr, while the other refs still print, exit 1.
  design §3.3 Exit code: the others still print and the exit code is 1; the repeat rule is as SHOW-DUP-01. Run: memex show nope.rs src/cli.rs nope.rs.

## LINE

- `SHOW-LINE-01` In text output a one-liner longer than 120 characters is cut to 119 characters and an ellipsis, so the line is 120 characters, exit 0.
  design §2 One-liners: text output truncates at 120 characters with an ellipsis. Run: memex show src/deep, whose child long.rs has a long first sentence.

## JSON

- `SHOW-JSON-01` `--json` on a file prints one document, `{"docs":[...],"errors":[]}`, whose doc has its ref, collection, id, path, title, text, empty fields, cites, cited_by and chain, exit 0.
  design §3.6 show shape. Run: memex show src/cli.rs --json.
- `SHOW-JSON-02` `--json` on a folder adds a `children` object with `total` and `items`, not counting the folder's own README.md, where each item has its `path`, its `kind` (`dir` or `file`) and its `title`, or null when it has none, exit 0.
  design §3.6 show: children. Run: memex show src --json.
- `SHOW-JSON-03` `--json` on a log ref gives a null `path`, the frontmatter as `fields` with lists as JSON arrays, and an empty `chain`, exit 0.
  design §3.6 Doc and show: logs add their frontmatter fields. Run: memex show impl:20261013-120000-show-walks-hub-readmes --json.
- `SHOW-JSON-04` `--json` with a ref that is not found prints the error in `errors` with code `not_found`, prints nothing on stderr, and exits 1.
  design §3.6 show shape and exit codes. Run: memex show nope.rs --json.
- `SHOW-JSON-05` `--json` on an ambiguous ref lists its `candidates` in the error, and exits 1.
  design §3.6 show shape (errors carry candidates) and design §3.1. Run: memex show README.md --json with cwd repo/src.

## NOW

- `SHOW-NOW-01` An invalid `$MEMEX_NOW` exits 3 for show too, though show prints no time: `invalid MEMEX_NOW:` and the value on stderr, or the `io` envelope on stdout with `--json`.
  design §3.6 Determinism: an invalid value exits 3, and the value is checked once as every verb starts. Run: memex show README.md with MEMEX_NOW as `2026-10-13 12:00:00`, then memex show README.md --json with `not-a-date`, then memex show README.md with the default value, which exits 0.

## OTHER

- `SHOW-OTHER-01` A file in another root, a sibling folder with its own `.memex/`, prints `cites` and `cited by` as `(in another root: run show from <root>)` in place of their lists, with no limits line, and its chain comes from that root; with `--json` its `cites` and `cited_by` are null, exit 0.
  design §3.1 Other worktrees and §3.6 show: cites and cited_by are null for an entry in another root. Run: memex show ../other/src/cli.rs from repo/, then the same with --json.
