---
memex: cases
---
# Info cases

## HEAD

- `INFO-HEAD-01` The first line is `memex`, the root as an absolute path, then `log store` with the store root-relative and ending in `/`, exit 0.
  Scenario: a root with a `.memex/` and one log entry; the header reads `memex  $SANDBOX/repo · log store .memex/logs/`. design §3.5 Header. Run: memex info.
- `INFO-HEAD-02` A store outside the root, as in a linked worktree, prints as its absolute path, and the streams are those of the main checkout's store, exit 0.
  Scenario: a worktree whose main checkout holds the log entry; run in the worktree, the header names the main checkout's `.memex/logs` and the streams line counts its entry. design §3.5 Header and §4.1 item 3.

## INTENT

- `INFO-INTENT-01` The intent line counts the built-in docs in the order readmes, headers and cases: the README of each folder, each file with a header, and each case id in a CASES.md, exit 0.
  Scenario: three READMEs, one file with a header and one without, and two case ids; the line reads `intent  3 readmes · 1 header · 2 cases`. design §3.5 Header and §3.1 built-in collections.
- `INFO-INTENT-02` Files the walk skips (hidden entries, paths named in `.ignore`, `node_modules` and `target`) are not counted.
  Scenario: a hidden folder's README, an ignored folder's README, a `node_modules` README and a `target` header are present; the line reads `intent  1 readme · 1 header · 0 cases`. design §4.1 item 2.
- `INFO-INTENT-03` A count of one is singular, so the line reads `1 readme · 1 header · 1 case`.
  Scenario: one README, one file with a header and one case id. design §3.5 gives no count of one; singular is a choice pinned here.

## COLL

- `INFO-COLL-01` The collections line lists each manifest with its doc count in alphabetical order by name, and a manifest with no docs counts 0.
  Scenario: manifests `doom` (two docs), `perf` (one doc) and `alpha` (its root folder is missing, so 0 docs); the line reads `collections  alpha 0 · doom 2 · perf 1`. design §3.5 Collections: alphabetical by name; §3.6 Determinism.
- `INFO-COLL-02` With no manifest, the collections line is left out, and an empty `.memex/collections/` holding only `.gitkeep` prints none.
  Scenario: a root whose `.memex/collections/` holds only `.gitkeep`; the output has no collections line. design §3.5 Collections; the line is shown only when there are collections.

## STREAM

- `INFO-STREAM-01` The streams line lists each stream with its entry count, newest entry first, marks the first stream with new entries `(K new in 24h)` and the later ones `(K new)`, and leaves out a stream with none.
  Scenario: `review` (one entry, 30 minutes old), `impl` (three entries, two in the last 24 hours, the newest 1 hour old) and `drift` (three entries, all older than 24 hours); the line reads `streams  review 1 (1 new in 24h) · impl 3 (2 new) · drift 3`. design §3.5 Streams and its sample.
- `INFO-STREAM-02` Streams whose newest entries have the same time are ordered by name.
  Scenario: `beta` and `alpha`, each with one entry stamped 11:00; the line reads `streams  alpha 1 (1 new in 24h) · beta 1 (1 new)`. design §3.5 Streams (most recent activity, then name).
- `INFO-STREAM-03` An entry exactly 24 hours old is new, and an entry one second older is not.
  Scenario: stream `edge` with entries stamped 2026-10-12T12:00:00Z and 2026-10-12T11:59:59Z under MEMEX_NOW 2026-10-13T12:00:00Z; the line reads `streams  edge 2 (1 new in 24h)`. design §3.5 Streams and §3.6 `$MEMEX_NOW`; the boundary is a choice pinned here.
- `INFO-STREAM-04` With a `.memex/` but no logs yet, the header still names `.memex/logs/` and the streams line is left out.
  Scenario: a root whose `.memex/` holds only `config.json`. design §3.5 Header and §4.1 item 3 (the store folder is created on the first log).

## TO

- `INFO-TO-01` With `--agent`, the to line counts every entry in the store whose `to` list holds the agent, in any stream, and names the search that finds them: `to cook-2`, then `2 messages → memex search to:cook-2`.
  Scenario: four entries in two streams, two addressed to cook-2 (one also to orchestrator), one addressed to orchestrator alone and one with no `to`. design §3.5 `to` line and §3.4 Agent id.
- `INFO-TO-02` The agent id comes from `$MEMEX_AGENT` when `--agent` is not given, and `--agent` wins over the variable.
  Scenario: with MEMEX_AGENT=cook-2 the line names cook-2; with the same variable and `--agent orchestrator` it names orchestrator. design §3.4 Agent id.
- `INFO-TO-03` The to line is left out when no agent id is known, and an agent with no entries still gets its line, at zero.
  Scenario: in the TO-01 store, `--agent nobody` prints `to nobody`, then `0 messages → memex search to:nobody`, and a plain `info` prints no to line. design §3.5 `to` line (shown including at zero).
- `INFO-TO-04` The agent id matches a `to` entry without regard to case, as the `to:` filter does.
  Scenario: `--agent COOK-2` in the TO-01 store counts the same two entries, and the line names the id as given: `to COOK-2`, then `2 messages → memex search to:COOK-2`. design §3.2 Filters (exact and case-insensitive).
- `INFO-TO-05` An agent id that `log` would refuse is a usage error with exit 2, whether it comes from `--agent` or `$MEMEX_AGENT`, and with `--json` the error is the usage envelope.
  Scenario: `--agent "bad id"`, `--json --agent "bad id"`, and MEMEX_AGENT=bad:one; each names the value, and the first names the info usage line. design §3.4 Agent id and §3.6 Fatal errors.

## HERE

- `INFO-HERE-01` From a folder below the root, the here line names the nearest README at or above the cwd by its folder and its one-liner.
  Scenario: cwd `src/deep`, which has no README, under a `src/README.md` whose one-liner is `The source folder holds the library.`; the line names the folder `src` and gives that one-liner after `—`. design §3.5 `here` line.
- `INFO-HERE-02` From the root itself the here line is left out, though the root has a README.
  Scenario: the cwd is the root, which has a README. design §3.5 `here` line (shown only when the cwd is not the root).
- `INFO-HERE-03` A one-liner in the here line is cut at 120 characters, the last of them `…`.
  Scenario: a `src/README.md` whose first sentence is 150 characters; the here line keeps its first 119 characters and ends with `…`, 120 characters in all. design §2 One-liners and §3.5 `here` line.
- `INFO-HERE-04` With no README at or above the cwd, the here line is left out.
  Scenario: a root with no README and a cwd in `src/`, which has none. design §3.5 `here` line.

## USE

- `INFO-USE-01` The use line is the last line and names the three commands it points to, exit 0.
  Scenario: a root with a `.memex/` and no docs; the lines are the header, the intent line and the use line, in that order. design §3.5 use line.

## CAP

- `INFO-CAP-01` Text over 3,000 bytes cuts the longest list line to its first items ending in `+N more`, until the whole output is under the cap.
  Scenario: 200 streams of one entry each, named 90 characters long so that the cut does not move with the length of the sandbox path (the header carries the root); the streams line shows its first 26 items and then `+174 more`, and the output is at most 3,000 bytes. design §3.5 Header and the cap (`+N more`).
- `INFO-CAP-02` (planned) Text of exactly 3,000 bytes is not cut, and 3,001 bytes is.
  design §3.5: text over 3,000 bytes is cut. Planned: the header carries the sandbox root, whose length differs by OS, so no golden can sit on the boundary on every OS. Until then it is pinned by the unit test `info_cap_02_*` in src/render/info.rs, which pads the root to 3,000 and 3,001 bytes.

## WARN

- `INFO-WARN-01` An invalid manifest is a warning on stderr, printed once, and is left out of the collections line, which still lists the valid manifests, exit 0.
  Scenario: `good.json` (valid, one doc), `bad.json` (an unknown key) and `wrong.json` (its name is not its file name); stdout reads `collections  good 1` and stderr holds each warning once. design §4.2 Invalid manifests and §3.5 Collections.
- `INFO-WARN-02` With `--json`, `warnings` lists the invalid manifest and the duplicate case id, and the same two lines are on stderr, each once.
  Scenario: `bad.json` plus two CASES.md files that both declare DUP-001; the JSON `warnings` array holds the manifest warning and `memex: warning: duplicate case id DUP-001 in CASES.md, sub/CASES.md`. design §4.2 and §3.6 info.

## NOROOT

- `INFO-NOROOT-01` With no `.memex/` and no git, info prints what it can for the cwd: the root is the cwd, the header reads `log store none (run memex init)`, no streams line appears, and the to line for the agent is at zero, exit 0.
  Scenario: a folder with a README and a file with a header, run with `--agent cook-2`, so the to line reads `0 messages`. design §4.1 item 1, §3.5 Header and the `to` line (shown including at zero).
- `INFO-NOROOT-02` With git but no `.memex/`, the root is the git toplevel, the store is none, and the here line still finds the nearest README, exit 0.
  Scenario: a committed repo with a README at the top and in `src/`, run in `src/`; the header names the toplevel, and the here line names the folder `src` with the one-liner `The source folder.`. design §4.1 item 1 (git toplevel fallback).

## JSON

- `INFO-JSON-01` `--json` prints one document with root, log_store, collections (the built-ins, then the manifests with kind, description and owner, then the streams with kind `log`, new_24h and newest), to and warnings, exit 0.
  Scenario: a root with one manifest that has a description and an owner, one stream entry addressed to cook-2, and `info --json --agent cook-2`. design §3.6 info; `null` kind for a built-in and RFC 3339 newest are choices pinned here.
- `INFO-JSON-02` With no store, `log_store` is null and the collections are the three built-ins only.
  Scenario: the NOROOT-01 folder with `info --json`. design §3.6 info; `null` is a choice pinned here.
- `INFO-JSON-03` The here one-liner in JSON is the whole text, not cut at 120 characters.
  Scenario: the HERE-03 folder with `info --json` from `src/`. design §2 One-liners (JSON keeps the full text).

## NOW

- `INFO-NOW-01` An invalid `$MEMEX_NOW` exits 3 for info, with `invalid MEMEX_NOW:` and the value on stderr and nothing on stdout.
  Scenario: a root with a `.memex/`, run with MEMEX_NOW as `2026-10-13 12:00:00` (no `T` and no `Z`), then as `not-a-date`, both exit 3; then with the default value, which exits 0. design §3.6 Determinism: an invalid value exits 3, and `$MEMEX_NOW` replaces the clock.
## NOTEXT
- `INFO-NOTEXT-01` `info` prints no log title or body text: its text and its `--json` hold names, counts and times, and never the words of a log.
  Scenario: a `quokka` log and a `zebra` log, each with its words in its title and its body; neither word appears in the text or the JSON output. design §3.5 (the atlas gives where things are, never their contents).

## ZERO

- `INFO-ZERO-01` With `--agent` and no entries in the store, the to line still prints, naming the agent and then `0 messages → memex search to:<agent>`, and the JSON `to` is `{"agent":"<agent>","count":0}`, exit 0.
  Scenario: a root with a `.memex/` and no log entries; run `info --agent x`, then `info --json --agent x`. design §3.5 `to` line (shown including at zero, so an agent can tell that it is wired up) and §3.6 info (`to` is present whenever an agent is known).
- `INFO-ZERO-02` `$MEMEX_AGENT` alone gives the same zero line in text and in JSON, with no `--agent` given.
  Scenario: the ZERO-01 root with MEMEX_AGENT=x; run `info`, then `info --json`. design §3.4 Agent id and §3.5 `to` line.
