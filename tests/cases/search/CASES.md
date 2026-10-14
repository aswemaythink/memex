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
  design §3.2 Filters (until S3.4 a field:value term is a literal word). Run: memex search std::io.

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

## TIE

- `SRCH-TIE-01` Docs with equal scores are ordered by label, so `src/aa.rs` comes before `src/zz.rs`.
  design §3.2 (ranking breaks ties by ref) and §3.6 determinism. Run: memex search shared.

## LINE

- `SRCH-LINE-01` In text, a one-liner longer than 120 characters is cut to 119 characters and an ellipsis, while the JSON snippet keeps all of it.
  design §2 One-liners (text cuts at 120 characters with an ellipsis; JSON keeps the full text). Run: memex search widget, then memex search widget --json.

## FILTER

- `SRCH-FILTER-01` (planned) A `field:value` term such as `tag:show` keeps only the docs whose field matches exactly, ignoring case, and several filters combine with AND.
  design §3.2 Filters `field:value`. Run: memex search tag:show.

## ROSTER

- `SRCH-ROSTER-01` (planned) `--in cases` with no words prints the roster: each case's one-liner with its `<-N` cited-by count, and the default limit of 50.
  design §3.2 Filters without words and the roster. Run: memex search --in cases.

## SINCE

- `SRCH-SINCE-01` (planned) `since:2d` keeps only the docs that have a time, and `since:` takes `30m`, `6h`, `2d`, `1w` or a date.
  design §3.2 Reserved filters (since:). Run: memex search since:2d.
