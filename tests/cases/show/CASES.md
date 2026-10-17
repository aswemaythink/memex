---
memex: cases
---
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
  design §3.3: show . is the map of the root; design §3.1: `.` is the root. Run: memex show . in the root.
- `SHOW-ROOT-02` Without a `.memex/` folder, show still reads the intent layer from the cwd and prints the same root section, exit 0.
  design §4.1 item 1 and §3.5b: reads work without init. Run: memex show . in a repo with no .memex/.
- `SHOW-ROOT-03` `show .` run from the subfolder `repo/sub` prints the root map under its `=== . ===` line, not the subfolder's, exit 0.
  design §3.1 Resolution order, step 2: `.` is the root's own label, so it always means the root. Run: memex show . with cwd repo/sub.

## LOG

- `SHOW-LOG-01` A log ref shows its body under its `===` line, then a `fields` block with the frontmatter in the order the file gives it (lists as `[a, b]`), then cites and cited by, and no chain, exit 0.
  design §3.3 Log or collection doc, and §4.3 frontmatter. Run: memex show 20261013-120000-show-walks-hub-readmes.
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
  design §3.3 (a section per ref, separated by `===` lines). Run: memex show src/cli.rs notes.txt.

## ERR

- `SHOW-ERR-01` A ref that is not found prints `memex: not found: <ref> (find refs with memex search <words>)` on stderr while the other refs still print, and the exit code is 1.
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
  design §3.3 gives a section for each ref in the order given; that a repeat prints once is a ruling, which §3.3 should state. Run: memex show src/cli.rs notes.txt src/cli.rs.
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
  design §3.6 Doc and show: logs add their frontmatter fields. Run: memex show 20261013-120000-show-walks-hub-readmes --json.
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

## ORDER

- `SHOW-ORDER-01` A folder lists its children sorted by name, files and folders together, and no sibling whose name starts with the folder's name leaks in: `show .` lists exactly a, a!, a-b, a.b and a0 in that order, `show a` lists only f.rs and x/, and `show a-b` lists only g.rs, in text and with `--json`, exit 0.
  design §3.3 Folder: the children are the folder's own entries, with its README.md left out; design §3.6 Determinism: lists are sorted, ties broken by ref. Run: memex show ., memex show a and memex show a-b, each with and without --json.

## GAP

- `SHOW-GAP-01` A folder with no README, holding a Markdown file with a heading and a subfolder with no README, prints `(no README)` as its body and on the subfolder's line, the Markdown file's heading on its line with no placeholder, and after its children only the legend line `(no README): add README.md here; line 1 is # Name, then a paragraph saying what the folder is`, once, with no `(no header)` line, exit 0; with `--json` the same folder has no legend text, and the Markdown file's heading is its child's `title`, exit 0.
  design §3.3 Gaps: a section that prints a placeholder follows its children with one legend line per placeholder, once; JSON is unchanged. A Markdown child prints its title, never a placeholder (SHOW-MDCHILD-04). Run: memex show plain, then memex show plain --json.
- `SHOW-GAP-02` A headerless file prints `(no header)` as its body, and the `(no header)` legend line directly after that body, before `cites`, exit 0; with `--json` it has no legend text, exit 0.
  design §3.3 Gaps: a file section's legend follows its body. Run: memex show notes.txt, then memex show notes.txt --json.
- `SHOW-GAP-03` A folder with a README whose children all have a one-liner, and a file with a header, print no legend line, and `--json` has none either, exit 0.
  design §3.3 Gaps: a section that prints no placeholder gets no legend line. Run: memex show headed, memex show src/cli.rs --up 0, then memex show headed --json.

## MDCHILD

- `SHOW-MDCHILD-01` A Markdown child of a folder lists its first heading as its one-liner, in text and as the child's `title` in `--json`, exit 0.
  design §3.3 Folder: a Markdown file's child line shows its title, the §4.2 title rule. Run: memex show notes, then memex show notes --json.
- `SHOW-MDCHILD-02` A Markdown child with no heading lists its first non-blank line, trimmed, and its JSON title is that same text, exit 0.
  design §3.3 Folder and §4.2 title rule: a text with no heading gives its first non-blank line. Run: memex show notes, then memex show notes --json.
- `SHOW-MDCHILD-03` A `memex:`-keyed Markdown doc and a CASES.md entry index each list their own first heading, read after their frontmatter, exit 0.
  design §3.3 Folder: the title is read after any frontmatter (§2.2). Run: memex show notes.
- `SHOW-MDCHILD-04` An empty Markdown child lists its name alone with an empty JSON title, and a folder whose only child lacking a one-liner is that file prints no `(no header)` legend line, exit 0.
  design §3.3 Folder: an empty Markdown file shows its name alone; no placeholder, so no legend line. Run: memex show scraps, then memex show scraps --json.

## COMMENTS

- `SHOW-COMMENTS-01` `show src/a.rs --comments` on a file with a `//!` header prints the header, then `comments  (showing 3 of 3)` before `cites`, listing `  L8` (a `//` run with one blank line inside it is one block), `  L15` (the indented `// inner`) and `  L19` (a three-line `/* */` block), each text indented four spaces, and not the comment after code on L12, exit 0; with `--json` it gives `comments` with total 3, cut false and lines 8, 15 and 19.
  design §3.3 Comments: a block runs as the header does (§2.1), a comment after code is not read, and each row is `  L<n>` with its text indented four spaces. Run: memex show src/a.rs --comments, then the same with --json.
- `SHOW-COMMENTS-02` `show src/a.rs` without `--comments` prints the header and no comments list, and its `--json` has no `comments` key, because the header is preferred, exit 0.
  design §3.3 Comments (show prefers the header) and §3.6 show JSON. Run: memex show src/a.rs, then the same with --json.
- `SHOW-COMMENTS-03` A headerless `src/b.rs`, with code on L1, prints `(no header)` and its gap line, then, without a flag, `comments  (showing 3 of 5; --comments for all)` listing `  L3`, `  L6` and `  L10`, and its `--json` gives `comments` with total 5, those three items and cut false, exit 0.
  design §3.3 Comments (a file with no header prints its first 3 blocks without the flag, and its heading adds --comments for all) and §2.1 Header rule. Run: memex show src/b.rs, then the same with --json.
- `SHOW-COMMENTS-04` The same headerless `src/b.rs` with `--comments` lists all five blocks, `  L3`, `  L6`, `  L10`, `  L13` and `  L16`, under `comments  (showing 5 of 5)` after the `(no header)` gap line, exit 0.
  design §3.3 Comments (the list uses --limit, default 20, and the gap line still stands as the warning). Run: memex show src/b.rs --comments.
- `SHOW-COMMENTS-05` A headerless `src/c.rs` with no comment at all prints no comments list and, with `--json`, no `comments` key, and with `--comments` prints `comments  (showing 0 of 0)` then `  (none)`; `src/d.rs`, which has a header and no other comment, prints the same heading and `  (none)` with `--comments`, and its `--json` gives `comments` as `{"total":0,"cut":false,"items":[]}`, exit 0.
  design §3.3 Comments (an empty list prints (none), and the no-header list is left out when there is no other block) and §3.6 show JSON. Run: memex show src/c.rs, then with --json and with --comments; memex show src/d.rs --comments, then with --json.
- `SHOW-COMMENTS-06` `show src/e.rs --comments --limit 2` lists the first two of four blocks, `  L6` and `  L9`, under `comments  (showing 2 of 4; --limit 4 for all)`, and `show src/b.rs --limit 2` with no flag lists `  L3` and `  L6` under `comments  (showing 2 of 5; --comments for all)`, exit 0.
  design §3.3 Comments (a list cut by --limit has its own heading; the no-header list shows 3, or --limit if lower). Run: memex show src/e.rs --comments --limit 2, then memex show src/b.rs --limit 2.
- `SHOW-COMMENTS-07` `show src/tool.py --comments` on a Python file lists only `  L12`, the `# note` block after the header, and not the shebang on L1, the pragma on L2, the license block on L3 and L4, the header on L6 and L7, or the function docstring on L15, exit 0.
  design §3.3 Comments (the blocks listed are those after the header, or after the lines §2.1 skips; a docstring other than the module one is not a comment) and §2.1 Header rule. Run: memex show src/tool.py --comments.
- `SHOW-COMMENTS-08` `--comments` changes nothing where there are no comment blocks: `show plain`, `show notes.md` (Markdown with an HTML comment in it) and `show notes.txt` (an unknown extension) print the same with and without the flag, and `show src/a.rs --comments --up 0` still prints its list, `  L6`, exit 0.
  design §3.3 Comments (--comments is ignored on a folder, an entry or a log, and on a file with no comment family; the list sits before cites with --up 0). Run: memex show plain, then with --comments; memex show notes.md and memex show notes.txt, each with and without --comments; memex show src/a.rs --comments --up 0.
- `SHOW-COMMENTS-09` `show src/big.rs --comments` on a file of more than 1 MiB lists only `  L4`, the `// early` block, then ends the list with `… (comments cut at 1 MiB)`, never reaching the `// late` block on L115006, and its `--json` gives `cut` true, exit 0.
  design §3.3 Comments (show reads the whole file up to 1 MiB, and a file cut there ends the list with the cut line) and §3.6 show JSON. The fixture is 1,150,308 bytes. Run: memex show src/big.rs --comments, then the same with --json.
- `SHOW-COMMENTS-10` `show src/run.sh --comments` on a `.sh` file with a shebang and a `#` header lists only `  L7`, the `#` block after the code, and not the shebang on L1 or the header on L2 and L3, exit 0; with `--json` its `comments` has total 1 and that one item, exit 0.
  design §3.3 Comments (the blocks listed are those after the header, or after the lines §2.1 skips) and §2.1 Header detection (the `#` family; a shebang is skipped). Run: memex show src/run.sh --comments, then the same with --json.
- `SHOW-COMMENTS-11` The `--` family: `show src/schema.sql --comments` on a `.sql` file with a `--` header lists only `  L9`, the later `--` run, and not the header on L1 and L2; `show src/Main.hs --comments` on a `.hs` file, whose `{-#` pragma on L1 is skipped, lists only `  L10`, its `{- -}` block after the code, as one block of two lines; with `--json` each file gives that one item, exit 0.
  design §2.1 Header detection (the `--` family covers sql and hs, `{- -}` is a block, and a `{-#` pragma is skipped before a header) and §3.3 Comments (a block runs as the header does). Run: memex show src/schema.sql --comments, then the same with --json; memex show src/Main.hs --comments, then the same with --json.
- `SHOW-COMMENTS-12` `show src/page.html --comments` on an `.html` file whose `<!DOCTYPE html>` line is skipped and whose header is an `<!-- -->` comment lists only `  L9`, the later multi-line `<!-- -->` block that runs from L9 to L12, as one block of two text lines, and not the header on L2 to L5, exit 0; with `--json` its `comments` gives that one item with `line` 9, exit 0.
  design §2.1 Header detection (the `<!-- -->` family; a markup prologue is a pragma) and §3.3 Comments (a block runs as the header does). Run: memex show src/page.html --comments, then the same with --json.
- `SHOW-COMMENTS-13` `show src/tool.ml --comments` on an OCaml file lists only `  L6`, its `(* *)` block after the code, as one block of two lines, and `show src/tool.lua --comments` on a Lua file lists only `  L8`, its `--[[ ]]` block after the code, as one block of two lines; both exit 0.
  design §2.1 Header detection (the `(* *)` family for ml, the `--[[ ]]` block for lua) and §3.3 Comments (a block runs as the header does). Run: memex show src/tool.ml --comments, then memex show src/tool.lua --comments.
- `SHOW-COMMENTS-14` `show src/tool.py --comments` on a Python file with a `#` header, a blank line, then a module docstring before any code lists `  L4`, the docstring as two text lines, and `--json` gives that item with `line` 4; `show src/lib.rs --comments` lists `  L8`, a `//` block with a blank line inside it, which prints that blank line as an empty row, exit 0.
  design §3.3 Comments (a docstring before any statement, after a `#` header, is the module docstring and is listed; a blank line inside a block prints empty) and §3.6 show JSON. Run: memex show src/tool.py --comments, then the same with --json; memex show src/lib.rs --comments.
- `SHOW-COMMENTS-15` `show src/tool.py --comments` skips the triple-quoted `TEMPLATE` string whole, though its lines on L7 and L8 start with `#`, and lists only `  L11`; `show src/main.go --comments` lists `  L6`, the pragma `//go:build linux` after the code, printed as `go:build linux` with its note, and `  L11`, a run that starts with a bare `//` and is listed at that line, exit 0.
  design §3.3 Comments (any other triple-quoted string is skipped whole; pragma-like lines after the header that the family reads as comments are listed) and §2.1 Header detection (a block is a run of comment lines). Run: memex show src/tool.py --comments, then memex show src/main.go --comments.
- `SHOW-COMMENTS-16` `show src/crlf.rs --comments` on a `.rs` file with CRLF line ends lists the same blocks at the same lines as its LF twin `show src/lf.rs --comments`, `  L8` and `  L15`, with no `\r` in any text row, and its `--json` gives the same `line` and `text` values with no `\r`, exit 0.
  design §4.4 Text (CRLF reads as LF) and §3.3 Comments (a block is printed as its lines, markers removed). Run: memex show src/lf.rs --comments, then memex show src/crlf.rs --comments, then the same with --json.
- `SHOW-COMMENTS-17` `show src/agree.rs --comments` lists three blocks, `  L8`, `  L13` (its blank line inside printed as an empty row) and `  L20`, and its `--json` gives `comments` with total 3, cut false and the same three `line` and `text` values, exit 0 in both.
  design §3.6 show JSON (`comments` is `{total, cut, items:[{line, text}]}`) and §3.3 Comments (JSON `line` counts from 1 and `text` is the block as printed, unindented). Run: memex show src/agree.rs --comments, then the same with --json.
