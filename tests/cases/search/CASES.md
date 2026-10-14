# Search cases

## RANK

- `SRCH-RANK-01` Any word may match, and a doc that matches both words ranks above a doc that matches one of them.
  design §3.2 Words; PLAN §4 BM25F. Run: memex search apples bananas (two READMEs, one with both words).
- `SRCH-RANK-02` A rarer word counts for more: a doc that holds only a word few docs hold ranks above docs that hold only a common word, which tie and rank by label.
  design §3.2 Words (more matches rank higher); PLAN §4 BM25F idf. Run: memex search apples kiwi.
- `SRCH-RANK-03` A query that matches nothing prints nothing and exits 0.
  design §3.2 Words; design §3.6 exit codes (0 includes zero hits). Run: memex search zzqq.

## WEIGHT

- `SRCH-WEIGHT-01` A word in a doc's id outranks the same word in its body, so a header whose path holds the word comes before one that only mentions it.
  design §3.2 Words; PLAN §4 field weights (id 4, title 3, body 1). Run: memex search zephyr.
- `SRCH-WEIGHT-02` A header is indexed only through its first 60 lines, so a word on line 61 of a header is not found, while the same word on line 70 of a README is.
  design §2.1 Limits (the index keeps the first 60 lines) and §3.2. Run: memex search marker.

## STEM

- `SRCH-STEM-01` A word matches its stem-mates, so `walking` finds a doc that says `walks` and leaves out a doc that shares no stem.
  design §3.2 Words (English stemming); PLAN §4 (exact or stem, multiplier 1.0). Run: memex search walking.

## PREFIX

- `SRCH-PREFIX-01` A word of 3 or more characters matches the start of a longer word, so `wal` finds a doc that says `walrus`.
  design §3.2 Words (prefix matching); PLAN §4 (prefix multiplier 0.5 for query words of 3 or more characters). Run: memex search wal.
- `SRCH-PREFIX-02` A word of 2 characters does not match by prefix, so `wa` finds nothing although `walrus` begins with it.
  design §3.2 and PLAN §4 (prefix only for query words of 3 or more characters). Run: memex search wa.

## FUZZY

- `SRCH-FUZZY-01` A word of 5 or more characters matches a word one edit away, the hit is marked `[~variant]`, and it ranks below an exact match.
  design §3.2 Words (fuzzy: one edit for words of 5 or more, marked `[~variant]`); PLAN §4 (fuzzy multiplier 0.3). Run: memex search grammar.
- `SRCH-FUZZY-02` A word of 4 characters does not match one edit away, so `hasp` finds nothing although `hash` is one substitution away.
  design §3.2 and PLAN §4 (fuzzy only for query words of 5 or more characters). Run: memex search hasp.

## TOKEN

- `SRCH-TOKEN-01` An id is a whole token: `NAME-001` finds `cases:NAME-001` and not `cases:NAME-002`, while the word `name` finds both.
  design §3.2 Words (ids are kept as whole tokens, so NAME-001 matches exactly). Run: memex search NAME-001, then memex search name.
- `SRCH-TOKEN-02` A path is a whole token: `src/hash.rs` finds that header and not `other/hash.rs`, while the word `hash` finds both.
  design §3.2 Words (paths are kept as whole tokens). Run: memex search src/hash.rs, then memex search hash.
- `SRCH-TOKEN-03` A term with `::` is searched as its words, so `std::io` finds the README that holds `std` and `io`.
  design §3.2 Filters (a `field:value` term that no doc in scope has a field for is searched as words). Run: memex search std::io.

## GROUP

- `SRCH-GROUP-01` Groups come in the order readmes, headers, cases, manifest collections and then logs, and a group with no hit is left out.
  design §3.2 Grouped output. Run: memex search widget, then memex search gizmo.
- `SRCH-GROUP-02` All log streams form one logs group, and each log line shows its age, agent and branch, leaving out any part that is missing along with its separator.
  design §3.2 Grouped output (logs are one group) and the log line of design §3.2. Run: memex search widget.

## LIMIT

- `SRCH-LIMIT-01` Without `--in`, each group shows at most 5 hits and still says how many matched, and `--limit 2` shows 2 of the same group.
  design §3.2 Limits (per group, default 5). Run: memex search widget, then memex search widget --limit 2.

## IN

- `SRCH-IN-01` With `--in`, the default limit of each group is 20, so 22 matching cases show 20 of 22, where without `--in` they show 5.
  design §3.2 Limits (with --in the default is 20). Run: memex search widget, then memex search widget --in cases.
- `SRCH-IN-02` `--in logs` searches only the logs group, and `--in readmes,logs` searches those two groups in their usual order.
  design §3.2 (--in restricts the groups searched; logs means every stream). Run: memex search widget --in logs, then --in readmes,logs.

## JSON

- `SRCH-JSON-01` `--json` prints one document with the query, empty filters and the groups, and each hit carries its doc's ref, collection, id, path, title and fields, its score rounded to 3 decimals and its snippet, with a null path for a log.
  design §3.6 search shape. Run: memex search widget --json.
- `SRCH-JSON-02` `--json` lists a fuzzy hit's variant in `fuzzy`, and a search with zero hits prints the document with an empty groups list, exit 0.
  design §3.6 (stdout is always one JSON document; scores rounded to 3 decimals). Run: memex search grammar --json, then memex search zzqq --json.
- `SRCH-JSON-03` `--json` echoes the filters as an object with one list of values per name, in the order given, so `tag:show since:2d` gives the names `since` and `tag` with one value each.
  design §3.6 search shape (filters:{…}); design §3.2 Filters. Run: memex search tag:show since:2d --json.

## TIE

- `SRCH-TIE-01` Docs with equal scores are ordered by label, so `src/aa.rs` comes before `src/zz.rs`.
  design §3.2 (ranking breaks ties by ref) and §3.6 determinism. Run: memex search shared.

## LINE

- `SRCH-LINE-01` In text, a one-liner longer than 120 characters is cut to 119 characters and an ellipsis, while the JSON snippet keeps all of it.
  design §2 One-liners (text cuts at 120 characters with an ellipsis; JSON keeps the full text). Run: memex search widget, then memex search widget --json.

## FILTER

- `SRCH-FILTER-01` A `field:value` term such as `tag:show` keeps only the docs whose tags hold the value, exactly and ignoring case, and the logs it keeps are listed newest first with their age, agent and branch.
  design §3.2 Filters `field:value` and Filters without words. Run: memex search tag:show, then memex search TAG:SHOW.
- `SRCH-FILTER-02` Several filters combine with AND, so `agent:cook-2 tag:show` keeps only the docs that satisfy both.
  design §3.2 Filters (exact, case-insensitive, combined with AND). Run: memex search agent:cook-2 tag:show.
- `SRCH-FILTER-03` With words, the filters come first and the words rank only the docs they keep, so `widget tag:show` ranks the `show` logs and ignores the other logs.
  design §3.2 Words plus filters (filter, then rank). Run: memex search widget tag:show.
- `SRCH-FILTER-04` `to:<id>` finds the messages addressed to an id, because it matches an item of the `to` list: `to:cook-1` finds the logs that hold cook-1 in `to`, and no other log.
  design §3.2 Reserved filters (to: matches an item of the to list). Run: memex search to:cook-1.
- `SRCH-FILTER-05` `branch:` matches the branch exactly, and `worktree:` matches a worktree by its basename or, when the value has a `/`, by its full path.
  design §3.2 Reserved filters (branch:, worktree: basename or path). Run: memex search branch:feat/show, then worktree:wt-cook-3, then worktree:/work/wt-cook-2.
- `SRCH-FILTER-06` A filter with no words lists its docs in the same groups and with the same limit as a search: `--limit 1` shows one of the four `show` logs and says it is one of four.
  design §3.2 Filters without words (same grouping and limits). Run: memex search tag:show, then memex search tag:show --limit 1.

## LITERAL

- `SRCH-LITERAL-01` A `field:value` term whose field no doc in scope holds is searched as its words, so `zzq:widget` finds the READMEs that say `widget`, the word `zzq` being in none of them.
  design §3.2 Filters (any other `x:` is a filter only if some doc in scope has field `x`; otherwise it is a literal word). Run: memex search --in readmes zzq:widget.
- `SRCH-LITERAL-02` `cases:WID-ONE-01` is not a filter, since no doc holds a field `cases`, so it is searched as words and its id ranks WID-ONE-01 first.
  design §3.2 Filters (a literal `cases:X` stays searchable). Run: memex search cases:WID-ONE-01 --in cases --limit 2.
- `SRCH-LITERAL-03` Which fields count depends on the groups searched: `stream:impl` filters the logs when no `--in` is given, but with `--in readmes` no doc in scope holds `stream`, so the term is words.
  design §3.2 Filters (some doc in scope has field x). Run: memex search stream:impl, then memex search --in readmes stream:impl.

## RESERVED

- `SRCH-RESERVED-01` A reserved name is always a filter, even where no doc in scope holds its field, so `agent:cook-2` in `--in readmes` matches nothing and exits 0, while the word `agent` would find a README.
  design §3.2 Filters (reserved: agent:, to:, tag:, branch:, worktree:, since:). Run: memex search --in readmes agent:cook-2.

## ROSTER

- `SRCH-ROSTER-01` `--in cases` with no words prints the roster: every case's one-liner ending in its `<-N` cited-by count, grouped with the default limit of 50, so 52 cases show 50 of 52.
  design §3.2 Roster (--in with no words; default limit 50). Run: memex search --in cases.
- `SRCH-ROSTER-02` The roster of logs lists them newest first by `time`, then by label, and each line keeps its age, agent and branch before its `<-N`.
  design §3.2 Roster (logs newest first) and the log line of §3.2. Run: memex search --in logs.
- `SRCH-ROSTER-03` A roster takes filters, so `--in logs tag:show` lists only the logs that hold the tag, each with its `<-N`.
  design §3.2 Roster (filters allowed). Run: memex search --in logs tag:show, then memex search --in logs to:cook-1.
- `SRCH-ROSTER-04` The `--json` roster gives each hit `score: null` and `cited_by` with its direct count, and `--limit` sets how many hits each group shows.
  design §3.6 search shape (roster hits have score null and cited_by N). Run: memex search --in cases --limit 2 --json.

## ORPHAN

- `SRCH-ORPHAN-01` In a roster, `<-N` counts the docs that cite a doc directly, so a case that nothing cites reads `<-0`, one cited by one doc reads `<-1`, and one cited by two docs reads `<-2`.
  design §3.2 Roster (each line carries its cited-by count) and design §2.3 (cited by). Run: memex search --in cases --limit 5.
- `SRCH-ORPHAN-02` A prefix citation is not counted in `<-N`, so a case that only the root README reaches through `WID-*` reads `<-0`, while `memex show` lists it as cited by `WID-*`.
  design §2.3 (a prefix citation is excluded from roster `<-N`). Run: memex search --in cases --limit 5, then memex show cases:WID-ONE-04.

## SINCE

- `SRCH-SINCE-01` `since:2d` keeps only the docs that have a time within two days before `$MEMEX_NOW`, and leaves out the docs that have none.
  design §3.2 Reserved filters (since: keeps only docs that have a time). Run: memex search since:2d.
- `SRCH-SINCE-02` `since:` takes `30m`, `6h`, `1w` or a `YYYY-MM-DD` date, and a date is the UTC midnight at the start of that day.
  design §3.2 Reserved filters (since: forms). Run: memex search since:30m, then since:6h, since:1w, then since:2026-10-12.
- `SRCH-SINCE-03` The cutoff is inclusive: a log whose time is exactly the cutoff passes, so `since:1d` keeps the log at 12:00 on the day before `$MEMEX_NOW`.
  design §3.2 (docs at or after the cutoff pass). Run: memex search since:1d.
- `SRCH-SINCE-04` A `since:` value that is none of the forms is a usage error: exit 2, with the usage line, and with `--json` a JSON error on stdout.
  design §3.2 Reserved filters (invalid since: is a usage error, exit 2) and design §3.6 exit codes. Run: memex search since:yesterday, then memex search since:2026-02-30 --json.
