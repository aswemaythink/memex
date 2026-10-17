---
memex: cases
---
# CLI cases

## VERSION

- `CLI-VERSION-01` `--version` prints `memex` and the crate version, exit 0.
  design §3. Run: memex --version.

## HELP

- `CLI-HELP-01` `--help` prints the top-level help on stdout, exit 0.
  design §3: the grammar block, then one line per verb. Run: memex --help.
- `CLI-HELP-02` `--help` after `search` prints the search help on stdout, exit 0.
  design §3 and §3.2 for its flags. Run: memex search --help.
- `CLI-HELP-03` `--help` after `show` prints the show help on stdout, exit 0, though no ref is given.
  design §3 and §3.3 for its flags. Run: memex show --help.
- `CLI-HELP-04` `--help` after `log` prints the log help on stdout, exit 0, though no stream is given.
  design §3 and §3.4 for its flags. Run: memex log --help.
- `CLI-HELP-05` `--help` after `info` prints the info help on stdout, exit 0.
  design §3 and §3.5 for its flags. Run: memex info --help.
- `CLI-HELP-06` `--help` after `init` prints the init help on stdout, exit 0.
  design §3 and §3.5b. Run: memex init --help.
- `CLI-HELP-07` `--help` after a verb ignores the rest, so `show --help --bogus` prints the show help and never reports `--bogus`.
  design §3 Flags: `--help` works after any verb and short-circuits: the arguments after it are not checked. Run: memex show --help --bogus.
- `CLI-HELP-08` After `--`, `--help` is a word, so `info -- --help` is an unexpected argument, exit 2.
  design §3 Flags: `--` ends flag parsing. Run: memex info -- --help.

## BARE

- `CLI-BARE-01` Bare `memex` prints the usage block on stderr, with no message line, and exits 2.
  design §3 Bare memex. Run: memex.
- `CLI-BARE-02` `search` with no words, filters or `--in` is a usage error that points to `memex info`, exit 2.
  design §3.2 No arguments. Run: memex search.

## VERB

- `CLI-VERB-01` An unknown verb is a usage error that names it and prints the whole grammar block, exit 2.
  design §3 grammar. Run: memex bogus.
- `CLI-VERB-02` `help` is not a verb, so `memex help` is an unknown verb, exit 2.
  design §3 grammar lists `memex --help | --version` and no help verb. Run: memex help.
- `CLI-VERB-03` A flag before any verb, with no verb after it, is an unknown flag and prints the whole grammar block, exit 2.
  design §3 Flags. Run: memex --frob.

## ARGS

- `CLI-ARGS-01` `show` with no refs is a usage error: it needs at least one path or ref, exit 2.
  design §3 grammar `<path|ref>...`. Run: memex show.
- `CLI-ARGS-02` `log` with no stream is a usage error: it needs a stream name, exit 2.
  design §3 grammar `<stream>`. Run: memex log.
- `CLI-ARGS-03` `log` takes one stream, so a second word is an unexpected argument, exit 2.
  design §3 grammar `<stream>`. Run: memex log impl extra.
- `CLI-ARGS-04` `info` takes no words, so `info x` is an unexpected argument, exit 2.
  design §3 grammar `memex info [--agent <id>] [--json]`. Run: memex info x.
- `CLI-ARGS-05` `init` takes no words, so `init x` is an unexpected argument, exit 2.
  design §3 grammar `memex init`. Run: memex init x.

## FLAG

- `CLI-FLAG-01` An unknown flag, one that no memex verb takes, is a usage error that names it, exit 2.
  design §3 Flags: unknown flags are errors and are never swallowed. Run: memex search words --frob.
- `CLI-FLAG-02` A flag given twice is a usage error that names it, exit 2.
  design §3 Flags: repeated flags are errors. Run: memex search words --limit 1 --limit 2.
- `CLI-FLAG-03` A memex flag that the verb's grammar line does not name is misplaced, so `init --json` is an error, exit 2.
  design §3 Flags: a flag with no effect is an error, judged on the verb's grammar. Run: memex init --json. The flag is `--json`, so the error prints as JSON.
- `CLI-FLAG-04` `--up` belongs to `show` only, so `search words --up 1` is a misplaced flag, exit 2.
  design §3 grammar and §3.3. Run: memex search words --up 1.
- `CLI-FLAG-05` `--version` is top-level only, so `search --version` is a misplaced flag, exit 2.
  design §3 grammar `memex --help | --version`. Run: memex search --version.
- `CLI-FLAG-06` A memex flag before the verb is misplaced, and the usage line that follows is the verb's, exit 2.
  design §3 Flags: flags belong after the verb. Run: memex --limit 5 search words.
- `CLI-FLAG-07` A flag that needs a value, given as the last argument, is a usage error that names it, exit 2.
  design §3 Flags: a value flag takes the next argument. Run: memex show ref --up.
- `CLI-FLAG-08` A flag is never a value, so `--up --limit 2` reports `--up` as missing its value, exit 2.
  design §3 Flags: flags are never swallowed. Run: memex show ref --up --limit 2.
- `CLI-FLAG-09` A count flag given a non-number is a bad value, exit 2.
  design §3 grammar `--limit N`. Run: memex search words --limit abc.
- `CLI-FLAG-10` `--up=-1` reads its value after the equals sign, and a negative count is a bad value, exit 2.
  design §3.3 `--up N`; counts are non-negative integers. Run: memex show ref --up=-1.
- `CLI-FLAG-11` A list flag with an empty item is a bad value, so `--in cases,` is an error, exit 2.
  design §3 Flags: lists are comma-separated in one flag value. Run: memex search words --in cases,.
- `CLI-FLAG-12` A switch given a value with an equals sign is a bad value, so `--json=yes` is an error, exit 2.
  design §3 grammar `[--json]`, which takes no value. Run: memex search words --json=yes. The flag is `--json`, so the error prints as JSON.
- `CLI-FLAG-13` After `--`, `--tag` is a word, so `log -- --tag x` takes `--tag` as the stream and `x` is an unexpected argument, exit 2.
  design §3 Flags: `--` ends flag parsing. Run: memex log -- --tag x.
- `CLI-FLAG-14` `show --up 1` with a ref that is not found is a not-found error, exit 1, and not a usage error.
  design §3.3 (`--up N` limits the depth) and §3.6 exit codes: 1 is not found. Run: memex show --up 1 nope.rs.

## JSON

- `CLI-JSON-01` With `--json` after the verb, an unknown flag is one JSON error line on stdout, stderr empty, exit 2.
  design §3: `--json` anywhere before `--` makes usage errors JSON. Run: memex search --json --frob.
- `CLI-JSON-02` The same JSON error holds when `--json` comes after the bad flag.
  design §3: `--json` anywhere before `--` makes usage errors JSON. Run: memex search --frob --json.
- `CLI-JSON-03` With `--json` before the verb, the misplaced-flag error is a JSON envelope, exit 2.
  design §3 and §3.6 fatal errors `{error:{code, message}}`. Run: memex --json search words.
- `CLI-JSON-04` After `--`, `--json` is a word, so an error about `init -- --json` stays text on stderr, exit 2.
  design §3: `--json` anywhere before `--` makes usage errors JSON. Run: memex init -- --json.
- `CLI-JSON-05` The JSON envelope escapes quotes, backslashes and control characters in its message, exit 2.
  design §3.6 fatal errors. Run: memex search --json with the flag `--bad"\` followed by a tab.

## EXIT

- `CLI-EXIT-01` A text usage error writes nothing to stdout and exits 2.
  design §3.6 exit codes: 2 is a usage error. Run: memex log impl --frob.
- `CLI-EXIT-02` `not_found` exits 1, and with `--json` prints its envelope on stdout.
  design §3.6 exit codes. Pinned by the REF family, where a ref does not resolve (design §3.1). Run: memex show nope.rs --json. The envelope is the show document of §3.6, whose errors entry has code not_found, since no verb prints a fatal not_found.
- `CLI-EXIT-03` `ambiguous` exits 1, and with `--json` prints its envelope on stdout.
  design §3.6 exit codes. Pinned by the REF family, where several refs match (design §3.1). Run: memex show README.md --json with cwd repo/src. The envelope is the show document, whose errors entry has code ambiguous and its candidates.
- `CLI-EXIT-04` `no_root` exits 3, and with `--json` prints its envelope on stdout.
  design §3.6 exit codes and §4.1 item 3. Run: memex log impl --json in a git repo with no `.memex/`, the setup of DISC-ROOT-04.
- `CLI-EXIT-05` `io` exits 3, and names the io::ErrorKind, never the OS text.
  design §3.6 exit codes: 3 for an I/O failure that stops the verb (a show ref that fails on I/O is that ref's error, exit 1, §3.3). Run: memex log impl with .memex/logs a regular file. Pinned by the PORT family, which also pins the JSON form.

## HINT

- `CLI-HINT-01` A ref that names nothing prints `memex: not found: <ref> (find refs with memex search <words>)` on stderr, with the ref as typed, and exits 1.
  design §3.5c Messages (not found names the next step). Run: memex show nope.rs.
- `CLI-HINT-02` With `--json`, the not-found ref's message in `errors` carries the same hint, stdout is the show document, stderr is empty, and exit is 1.
  design §3.5c Messages and §3.6 (the show document's errors). Run: memex show nope.rs --json.
- `CLI-HINT-03` An unknown `--in` name prints `memex: unknown collection '<name>'; collections: <names>` on stderr, then the usage line, and exits 2. The names are readmes, headers, the collections in name order (manifests and frontmatter alike), logs, then the streams in name order.
  design §3.5c Messages and §3.2 `--in` names. Fixture: a README, a frontmatter collection `notes` (its `notes/alpha.md` has `memex: notes`), and a log entry in stream `impl`. Run: memex search widget --in nope.
- `CLI-HINT-04` With `--json`, the unknown-collection message is the usage envelope on stdout, with the same names, stderr is empty, and exit is 2.
  design §3.5c Messages and §3.6 (usage errors are JSON with --json). Fixture as CLI-HINT-03. Run: memex search widget --in nope --json.
- `CLI-HINT-05` A blank log body prints `memex: log body is empty: memex log <stream> <<'EOF' … EOF` on stderr, then the log usage line, writes nothing, and exits 2.
  design §3.5c Messages and §3.4 Body. Run: memex log impl with stdin of blank lines.
- `CLI-HINT-06` With `--json`, the blank-body message is the usage envelope on stdout, stderr is empty, and exit is 2.
  design §3.5c Messages and §3.6. Run: memex log impl --json with stdin of blank lines.
- `CLI-HINT-07` A second word for `log` prints `memex: unexpected argument '<word>': log reads its body from stdin: memex log <stream> <<'EOF' … EOF` on stderr, then the log usage line, and exits 2.
  design §3.5c Messages (unexpected argument, with the log form). Run: memex log impl extra.
- `CLI-HINT-08` With `--json`, the log form of the unexpected-argument message is the usage envelope on stdout, and exit is 2.
  design §3.5c Messages and §3.6. Run: memex log impl extra --json.
- `CLI-HINT-09` A second word for `info` prints `memex: unexpected argument 'extra'` with no log form added, then the info usage line, and exits 2.
  design §3.5c Messages (the log form is for log alone). Run: memex info extra.
- `CLI-HINT-10` An unknown verb prints `memex: unknown verb '<word>'; run memex --help` on stderr, then the grammar block, and exits 2.
  design §3.5c Messages. Run: memex bogus.
- `CLI-HINT-11` With `--json` before the verb, the unknown-verb message is the usage envelope on stdout, and exit is 2.
  design §3.5c Messages and §3.6. Run: memex --json bogus.
- `CLI-HINT-12` (planned) A name that another process takes between init's check and its write prints `memex: cannot create <path>: another process created it while init ran, or a name that differs only in case holds it` on stderr, leaves that file as it is, and exits 3.
  design §3.5b (init race). Planned: a golden needs a concurrent writer or a case-insensitive file system, which no run can set up on every OS; the unit test in src/verbs/init/files.rs pins the text.

## HINTB

- `CLI-HINTB-01` `log` with no stream names its next step: stderr reads `memex: log needs a stream name: memex log <stream> <<'EOF' … EOF`, then the log usage line, and exits 2; with `--json` the same message is the usage envelope on stdout.
  design §3.5c Messages (log needs a stream name adds the heredoc form). Run: memex log, then memex log --json.
- `CLI-HINTB-02` A reserved stream name, `logs` for example, names its next step: stderr reads `memex: stream name '<name>' is reserved; pick another name, such as impl`, then the log usage line, and exits 2 with nothing written; with `--json` the same message is the usage envelope on stdout.
  design §3.5c Messages (stream name is reserved adds the next step). Run: memex log logs, then memex log --json readmes, in a root with a `.memex/`. `cases` is no longer reserved, so it is not used here.
