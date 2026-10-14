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
  design §3 Flags: `--help` works after any verb. Run: memex show --help --bogus.
- `CLI-HELP-08` After `--`, `--help` is a word, so `info -- --help` is an unexpected argument, exit 2.
  design §3 Flags: `--` ends flag parsing. Run: memex info -- --help.

## BARE

- `CLI-BARE-01` Bare `memex` prints the usage block on stderr, with no message line, and exits 2.
  design §3 Bare memex. Run: memex.
- `CLI-BARE-02` `search` with no words, filters or `--in` is a usage error that points to `memex info`, exit 2.
  design §3.2 No arguments. Run: memex search.

## VERB

- `CLI-VERB-01` An unknown verb is a usage error that names it and prints the whole grammar block, exit 2.
  design §3 grammar; PLAN §1 fix 14. Run: memex bogus.
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
  design §3.6 fatal errors; PLAN §1 fix 17. Run: memex search --json with the flag `--bad"\` followed by a tab.

## EXIT

- `CLI-EXIT-01` A text usage error writes nothing to stdout and exits 2.
  design §3.6 exit codes: 2 is a usage error. Run: memex log impl --frob.
- `CLI-EXIT-02` `not_found` exits 1, and with `--json` prints its envelope on stdout.
  design §3.6 exit codes. Pinned by the REF family, where a ref does not resolve (design §3.1). Run: memex show nope.rs --json. The envelope is the show document of §3.6, whose errors entry has code not_found, since no verb prints a fatal not_found.
- `CLI-EXIT-03` `ambiguous` exits 1, and with `--json` prints its envelope on stdout.
  design §3.6 exit codes. Pinned by the REF family, where several refs match (design §3.1). Run: memex show README.md --json with cwd repo/src. The envelope is the show document, whose errors entry has code ambiguous and its candidates.
- `CLI-EXIT-04` (planned) `no_root` exits 3, and with `--json` prints its envelope on stdout.
  design §3.6 exit codes. Pinned by the DISC family, where log finds no `.memex/` store (design §4.1).
- `CLI-EXIT-05` (planned) `io` exits 3, and names the io::ErrorKind, never the OS text.
  design §3.6 exit codes; PLAN §1 fix 17. Pinned by the PORT family.
