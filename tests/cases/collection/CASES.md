---
memex: cases
---
# Collection cases

## FRONT

- `COLL-FRONT-01` A Markdown file whose frontmatter has `memex: decisions` is one doc of `decisions`, labelled `decisions:<file stem>`, so `show decisions:0001-cache` prints its title, fields and text, exit 0.
  design §2.2 (frontmatter, one file one doc). Run: memex show decisions:0001-cache.
- `COLL-FRONT-02` A byte order mark before the frontmatter is dropped, so the file is still a `decisions` doc, exit 0.
  design §2.2 and §4.4 Text. Run: memex show decisions:0002-bom. The fixture starts with a byte order mark, then the frontmatter.
- `COLL-FRONT-03` A CRLF frontmatter gives its fields and a body with no carriage return, exit 0.
  design §2.2 and §4.4 Text. Run: memex show decisions:0003-crlf --json.
- `COLL-FRONT-04` A frontmatter whose fence never closes is no frontmatter, so its file is no doc: `show decisions:0004-open` is not found, exit 1, with no warning.
  design §2.2 (the closing line). Run: memex show decisions:0004-open.
- `COLL-FRONT-05` A `memex:` key past the first 8 KiB of the file is not seen, so the file is no doc and `show decisions:0005-late` is not found, exit 1.
  design §2.2 (the key counts within the first 8 KiB). Run: memex show decisions:0005-late. A `pad:` line of 9,000 bytes comes before the key.
- `COLL-FRONT-06` A `memex:` line that the 8 KiB limit cuts is not seen, so `show decisions:0006-cut` is not found, exit 1.
  design §2.2 (the key counts within the first 8 KiB; the whole line must lie inside it, a ruling not yet in PLAN §13). Run: memex show decisions:0006-cut. The key starts at byte 8186, so the limit falls inside `memex: decisions`.
- `COLL-FRONT-07` A `memex:` key within the first 8 KiB makes a doc even when the file is longer than 8 KiB, and its text is the whole body, exit 0.
  design §2.2 and §2.1 (files read). Run: memex show decisions:0007-long. The key is at byte 4 and the closing fence comes after one 8,200-byte blank line, so the file is 8,266 bytes.
- `COLL-FRONT-08` A Markdown file with no frontmatter is no doc, even when its body has a `memex:` line, so `show decisions:0008-body` is not found, exit 1.
  design §2.2 (the file's first line is `---`). Run: memex show decisions:0008-body.

## NAME

- `COLL-NAME-01` A `memex:` value that is not a collection name (uppercase, a leading dash, a space, or 41 characters) skips its file with one warning each, so `info` lists no collection from them, exit 0.
  design §2.2 Name. Run: memex info. Each warning reads `memex: warning: <path>: 'memex: <value>' is not a collection name; use a-z, 0-9 and -`.
- `COLL-NAME-02` A 40-character name is a valid collection name, so `show <name>:x` prints its doc, exit 0.
  design §2.2 Name (`[a-z0-9][a-z0-9-]{0,39}`). Run: memex show abcdefghijklmnopqrstuvwxyz0123456789abcd:x.

## ONE

- `COLL-ONE-01` A file's `id` field is its id, so its doc is `decisions:ADR-001`, and its file stem is no id, so `show decisions:0001-cache` is not found, exit 1.
  design §2.2 (one file, one doc: the id field, else the file stem). Run: memex show decisions:ADR-001 decisions:0001-cache.
- `COLL-ONE-02` Without an `id` field the file stem is the id, and the title is the first heading outside a code fence, so `search --in notes` lists `notes:retry  Retry policy`, exit 0.
  design §2.2 (one file, one doc) and §4.2 title. Run: memex search --in notes. The fixture fences a `# Not a title` heading first.
- `COLL-ONE-03` A doc with no heading takes its first line with text as its title, so `search --in notes` lists `notes:plain` with that line, exit 0.
  design §4.2 title (the default line when there is no heading). Run: memex search --in notes.
- `COLL-ONE-04` A doc's fields are its frontmatter keys other than `memex`, in the order the file gives them, and `show` prints them, exit 0.
  design §2.2 (one file: fields minus memex) and §3.3 (a collection doc). Run: memex show decisions:0004-fields.

## ENTRY

- `COLL-ENTRY-01` A column-0 item that opens with a case id is an entry of its file's collection, labelled `perf:PERF-001`, and the file is no doc, so `show perf:budgets` is not found, exit 1.
  design §2.2 (entries). Run: memex show perf:PERF-001 perf:budgets.
- `COLL-ENTRY-02` An entry's group is the nearest heading above it, and an entry above every heading has no group, exit 0.
  design §2.2 (group). Run: memex show perf:PERF-010 perf:PERF-011 perf:PERF-012.
- `COLL-ENTRY-03` The entry index's frontmatter keys are not copied to its entries, so `show perf:PERF-020` prints no `status` or `owner`, exit 0.
  design §2.2 (group: the file's frontmatter keys are not copied). Run: memex show perf:PERF-020.
- `COLL-ENTRY-04` An item inside a code fence is no entry, so a file whose only id-led item is fenced is one doc, `perf:fenced`, and `show perf:PERF-030` is not found, exit 1.
  design §2.2 (entries outside fenced code) and §2.3 (fences). Run: memex show perf:fenced perf:PERF-030.

## PLAIN

- `COLL-PLAIN-01` A CASES.md without the `memex:` key is no collection: its items are no cases, so `show NAME-001` is not found, exit 1.
  design §2.2 (a file with no `memex:` key is no doc). Run: memex show NAME-001.

## README

- `COLL-README-01` A README whose frontmatter holds `memex:` keeps its graph's doc, the key is ignored with the README warning, and `show docs/README.md` prints it, exit 0.
  design §2.2 (a README is the graph's, and its key is ignored). Run: memex show docs/README.md.

## UNREAD

- `COLL-UNREAD-01` A Markdown file that cannot be read gives one warning, `cannot read <path> (PermissionDenied)`, and is skipped, so the readable `notes:open` still prints, exit 0.
  design §4.2 (unreadable files: a Markdown file is a warning). Run: memex show notes:open. The fixture's notes/locked.md has mode 000; root reads it, so the case is an ignored trial there.

## CITE

- `COLL-CITE-01` A bare id of an entry in another collection is cited: the README cites `perf:PERF-001`, and the entry lists the README under cited by, exit 0.
  design §2.3 (a bare id that is declared somewhere). Run: memex show README.md perf:PERF-001.
- `COLL-CITE-02` A `collection:id` ref in a README resolves to a whole-file doc, so the README cites `decisions:0001-cache`, exit 0.
  design §2.3 (`collection:id`). Run: memex show README.md.
- `COLL-CITE-03` A link to a whole file's path cites its doc: the README cites `decisions:0001-cache`, and the file's own show lists the README under cited by, exit 0.
  design §2.3 (`[text](path)`) and §3.3 (a file's citations are its doc's). Run: memex show README.md decisions/0001-cache.md.
- `COLL-CITE-04` A path token in a whole file resolves from that file's folder, so `notes:a` cites `notes:sibling` through `sibling.md` beside it, exit 0.
  design §2.3 (a path token resolves from the citing doc's folder first). Run: memex show notes:a.

## INFO

- `COLL-INFO-01` A collection that only frontmatter names is counted on info's collections line, and its JSON kind is `markdown` with `builtin` false, exit 0.
  design §3.5 (collections) and §3.6 (info). Run: memex info; memex info --json.
- `COLL-INFO-02` A manifest and frontmatter files that name one collection form one collection: its count is both sources, and info shows the manifest's kind and description, exit 0.
  design §4.2 (one collection, two sources). Run: memex info --json.
