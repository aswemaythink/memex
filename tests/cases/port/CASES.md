---
memex: cases
---
# PORT cases

## CRLF

- `PORT-CRLF-01` A README with CRLF line ends gives the same folder section, file section and chain as its LF twin, and no output has a carriage return.
  design §4.4 Text: accept `\r\n` everywhere. Run: memex show docs, memex show docs/README.md and memex show docs --json, each from a CRLF root and from its LF twin, where the root README is in the chain.
- `PORT-CRLF-02` A Rust file whose `//!` header has CRLF line ends gives the same header and one-liner as its LF twin, with no carriage return.
  design §4.4 Text and §2.1 headers. Run: memex show src/cli.rs --up 0 and memex show src, each from a CRLF root and from its LF twin.
- `PORT-CRLF-03` A shell script whose `#` header has CRLF line ends, after a shebang, gives the same header as its LF twin, with no carriage return.
  design §4.4 Text and §2.1 pragma skip. Run: memex show tools/run.sh --up 0, from a CRLF root and from its LF twin.
- `PORT-CRLF-04` A CASES.md with CRLF line ends gives the same case as its LF twin, with no carriage return.
  design §4.4 Text and §2.2 case index. Run: memex show cases:CRL-TOPIC-01, from a CRLF root and from its LF twin.
- `PORT-CRLF-05` A Markdown manifest doc with CRLF frontmatter gives the same fields and body as its LF twin, with no carriage return.
  design §4.4 Text and §4.2 frontmatter. Run: memex show notes:alpha, from a CRLF root and from its LF twin.
- `PORT-CRLF-06` A log body piped in with CRLF line ends is stored with LF line ends, and its ref, its `show` text and its stored file match its LF twin.
  design §4.4 Text: write `\n`; §3.4 Body. Run: memex log impl with the CRLF body from a CRLF root, and with the LF body from its twin, then memex show each ref and snapshot both stores.

## SLASH

- `PORT-SLASH-01` `show` labels nested paths with `/`: the file's label, its folder's label, and each README in its chain.
  design §4.4 Paths and §3.1 labels. Run: memex show src/deep/cli.rs, then memex show src/deep.
- `PORT-SLASH-02` `search --json` gives each nested hit a `/`-separated `ref`, `id` and `path`.
  design §4.4 Paths and §3.6 search. Run: memex search deep --json.
- `PORT-SLASH-03` `log --json`, run from a nested folder, gives the entry's absolute path with `/` separators, and the entry is in the root's store.
  design §4.4 Paths and §3.6 log. Run: memex log impl --json from repo/src/deep.

## WRITE

- `PORT-WRITE-01` A folder at an entry's name and a folder at its `-2` name both count as taken, so the entry is written as `-3` and both folders stay as they were.
  design §4.4 Atomic writes and §4.3 concurrency. LOG-WRITE-08 pins the plain `-2`, and LOG-WRITE-15 pins the folder at the first name. Run: memex log impl over a stream that holds those two folders.

## DECODE

- `PORT-DECODE-01` A header with a byte that is not UTF-8 shows that byte as U+FFFD, in text and in JSON, exit 0.
  design §4.4 Encoding: non-UTF-8 content is decoded lossily. Run: memex show src/bad.rs --up 0, then the same with --json.
- `PORT-DECODE-02` A README with a byte that is not UTF-8 shows that byte as U+FFFD in its one-liner and in its text, exit 0.
  design §4.4 Encoding. Run: memex show guide, then memex show guide --json, whose title is the one-liner.
- `PORT-DECODE-03` (planned) A file whose name is not UTF-8 is skipped by `show` and `search`, exit 0.
  design §4.4 Encoding: non-UTF-8 path names are skipped. Planned: APFS refuses such a name, so no fixture or setup step can create one on macOS. Until then the label rule is pinned by the walk.rs unit test.

## ERROR

- `PORT-ERROR-01` A store path blocked by a regular file gives the io error as JSON on stdout: memex's message, the io::ErrorKind and the path, with stderr empty and exit 3.
  design §3.6 fatal errors and exit codes; §4.4 Errors. Run: memex log impl --json, where .memex/logs is a regular file.
- `PORT-ERROR-02` A stream folder blocked by a regular file names that stream's path in the io error, leaves the file as it was, and exits 3.
  design §4.4 Errors: the failing path, never the OS text. Run: memex log impl, where .memex/logs/impl is a regular file.

## BOM

- `PORT-BOM-01` A CASES.md saved with a BOM and CRLF line ends is read: `info` counts its case, and `show` prints the case with no BOM and no carriage return, exit 0.
  design §4.4 Text: a leading BOM is dropped on read. Run: memex info, then memex show cases:BOM-TOPIC-01.
- `PORT-BOM-02` A JSON manifest and its data file saved with a BOM are read: `info` lists the collection with its count and no warning, and `show` finds the row, exit 0.
  design §4.4 Text and §4.2 kind json. Run: memex info, then memex show rows:R-1.

## NAME

- `PORT-NAME-01` `Readme.md` is not a README: its folder shows `(no README)`, and `info` counts no readme for it, exit 0.
  design §4.4 Name matching: special names are matched exactly against listings. Run: memex show caseV, then memex info.
- `PORT-NAME-02` `cases.md` is not a CASES.md: its item is not a case, so `show NAM-ITEM-01` is not found, exit 1, and `info` counts no case.
  design §4.4 Name matching and §2.2 case index. Run: memex info, then memex show NAM-ITEM-01.
- `PORT-NAME-03` A folder named `.Memex` is not the store: `log` finds no `.memex/` and exits 3 with the no_root message.
  design §4.4 Name matching (`.memex` is matched against listings) and §4.1 item 1. Run: memex log impl, in a folder that holds only `.Memex/`.
- `PORT-NAME-04` A `.memex/Config.json` is not the config on any OS: alone, its `forever` is ignored, so a 4-day-old entry is pruned at the 3-day default, with no warning and exit 0.
  design §4.4 Name matching (`config.json` is matched against the listing) and §4.1 item 4. Run: memex log impl over a 4-day-old entry, with .memex/Config.json holding keep_logs forever.
- `PORT-NAME-05` A `.memex/Collections/` folder is not the collections folder on any OS: `memex log notes` ignores the `notes.json` inside it and exits 0.
  design §4.4 Name matching (`collections/` is matched against the listing) and §3.4 Stream name. Run: memex log notes, with .memex/Collections/notes.json present.

## MODE

- `PORT-MODE-01` A new log entry is 0644 before the umask, so 0644 under the runner's umask; a golden cannot show a mode, so tests/entry_mode.rs pins it under several umasks, and this case pins the ref and the stored text.
  design §4.4 File modes (0644 before umask, not tempfile's private 0600). Run: memex log impl with a short body, then the store snapshot.

## EXIT

- `PORT-EXIT-01` `log` into a stream folder at mode 555 exits 3 with an io error that names the folder, on stderr in text and in `--json` on stdout, and writes nothing into the folder.
  design §3.6 exit codes (3 for an I/O failure that stops the verb) and §4.4 Errors (the failing path, never the OS text). Run: memex log impl, then memex log impl --json, with .memex/logs/impl at mode 555.
