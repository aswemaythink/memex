# memex — design

> Status: the authoritative behaviour spec for memex 0.1. Rulings made during the build are committed here and listed in the PR description; §8 records overrides of the original plan.

## 1. Thesis and principles

**memex is authored context compression.** Whoever knew the most compresses it at the moment they knew it best. A README compresses a folder, a header compresses a file, a case line compresses a test, a log compresses an agent's run, and a ref compresses a handoff. Together these form an *intent layer*: a language on top of the code that agents read to understand architecture, and check against the code to find drift. memex reads it back one level at a time: one-liner (`search`), then document (`show`), then chain (`show <path>`).

- **Markdown files and a CLI.** No server, daemon, auth, MCP, LLM, vectors or database. memex starts, reads plain files live, prints and exits.
- **Agent-agnostic.** Anything with a shell can use it. Plugins only wrap it.
- **Conventions over configuration.** The intent layer needs zero config: a folder's README and a file's header are found by where they are, and a frontmatter line puts any Markdown file in a collection. Manifests exist only for structures that are not Markdown.
- **memex reads intent, `rg` reads code.** memex never indexes code bodies, only the comment block at the top of a file.
- **Bounded output.** Every list prints `showing N of M`, and `--limit` works everywhere. Nothing is capped silently.
- **Logs are scaffolding; knowledge is authored.** A log is mechanical: what happened in a run, useful for a couple of working days, then junk. memex prunes it. Anything still true after that gets compressed into a README, a header or a collection doc by someone who understands it. Automated memory tools try to keep everything forever; memex says most of it was never worth keeping.
- **A feature earns its place** by removing friction an agent actually hit in a playtest.

### Lineage

The name is Vannevar Bush's. In "As We May Think" (*The Atlantic*, July 1945) he described the memex: a desk of microfilm where a reader links documents side by side into **trails**, annotates them, and hands them to colleagues. He predicted a profession of **trail blazers** who build useful paths through the shared record. The same essay imagines a forehead camera that photographs everything. That's automatic capture, eighty years early. The memex is the other idea: associations made on purpose by someone who understands the material. That one led to Engelbart, Nelson's hypertext and the web.

This memex takes Bush's side. Citations and backlinks are the trails; developers and agents are the trail blazers; logs are the camera's film, kept a couple of days and then thrown away. Org: `aswemaythink`.

## 2. The intent layer: the graph and its dimensions

The intent layer has two parts, tied together by citations.

- **The graph** is attached to the tree and found by *where* it is: a README per folder and a header per file. `show <path>` walks it.
- **Dimensions** cut across the tree and are found by *what* they are: cases, decisions, perf notes, playtests. Each is a collection, and any Markdown file joins one with a frontmatter line (§2.2). Anyone can start one; no config is needed.
- **Citations** link the two both ways (§2.3). **Logs** are the time dimension: scaffolding that expires (§4.3).

Five conventions:

1. **README.md per unit folder.** Line 1 is `# Name`. The first paragraph says what the folder is and why it exists; its first sentence is the one-liner shown everywhere.
2. **Header at the top of each non-trivial file.** It is the first comment block, and its first sentence states the file's purpose. That sentence is the file's one-liner.
3. **A frontmatter line joins a dimension.** `memex: <collection>` in a Markdown file's frontmatter makes it a doc in that collection (§2.2). Cases are the usual first one: a `CASES.md` with `memex: cases` lists test ids with their claims, and each test carries its id, in its name where the language allows, else in a comment on it.
4. **Cite, don't copy.** Write ids, paths and refs into READMEs, headers, docs and logs. memex derives links both ways from them. There are no mapping files.
5. **Intent ships with code.** A change to behaviour updates its header or README in the same commit.

**One-liners.** A README's one-liner is the first sentence of the first paragraph after the H1, or the H1 text if there is no paragraph; a closed frontmatter block atop a README is not part of that text (search skips it too, and so does `show`'s text, §3.3). A header's is the first sentence of the block. A sentence ends at `.`, `!` or `?` followed by whitespace or the end of the paragraph. Text output truncates a one-liner at 120 chars with `…`; JSON keeps the full text.

### 2.1 Header detection

The comment syntax is chosen by extension from a built-in table. Files with unknown extensions are not harvested; new extensions are added to the table by PR.

| Family | Extensions (non-exhaustive) |
|---|---|
| `//`, `///`, `//!`, `/* */`, `/** */` | rs c h cc cpp hpp cs java kt kts scala swift go js jsx mjs cjs ts tsx mts cts dart php css scss less zig proto groovy |
| `#` | py rb sh bash zsh fish pl r jl ex exs nix tf yaml yml toml ps1 cmake mk, `Makefile`, `Dockerfile` |
| `--`, `{- -}`, `--[[ ]]` | hs elm lua sql ada |
| `;` | clj cljs el lisp scm asm |
| `%` | erl hrl tex m |
| `<!-- -->` | html vue svelte xml |
| `(* *)` | ml mli |
| `"""` / `'''` module docstring | py (used if there is no leading `#` block) |

- **Header rule.** After blank lines, a shebang, encoding/pragma lines (`# -*-`, `//go:build`, `"use client"`, and markup prologues `<?xml …?>`, `<!DOCTYPE …>`, `<?php`) and one leading license block (a block containing `SPDX-License-Identifier` or starting with `Copyright`); a license block ends at its first blank line, so a header must be separated from it by one, the next comment block is the header. It ends at the first line that is neither comment nor blank.
- **Limits.** The index keeps the first 60 lines; `show` prints the whole block. A file with no header shows as `(no header)` in listings. That gap is itself a signal. Markdown files have no header; listings show their title (§3.3).
- **Files read.** Only the first 8 KiB of each file is read. Binary files are skipped. A header still open at the 8 KiB cut is shown with a final line `… (header cut at 8 KiB)`, never silently.

### 2.2 Collections: `memex: <name>`

A walked Markdown file (listed name ending in `.md`) whose frontmatter holds the key `memex` is a doc in the collection that key names. No manifest or config is needed. README.md is the exception: it is the graph's, so its `memex` key never makes it a doc. Instead the key marks the README's folder as a root of the manifest it names (§4.2 Roots). A README whose key names no manifest gives the warning `memex: warning: <path> is a README; its memex: key marks a root of '<name>', which has no manifest — add .memex/collections/<name>.json, or move that content into its own file`, and `audit` lists it (§3.5d kind 8).

```markdown
---
memex: decisions
status: accepted
---
# Logs live in the main checkout
Worktrees are removed when a slice lands, so their logs would go with them.
```

- **Frontmatter.** The file's first line is `---` (after a BOM), the flat YAML of §4.2 follows, and the next `---` line closes it. The `memex` key counts only within the file's first 8 KiB, the limit headers use (§2.1), and only when its whole line lies inside them.
- **Name.** `[a-z0-9][a-z0-9-]{0,39}`, as for streams (§3.4). A file with any other value is a warning, `memex: warning: <path>: 'memex: <value>' is not a collection name; use a-z, 0-9 and -`, and is skipped. A name shared with the graph or a stream warns, never breaks (§4.2 Names).
- **One file, one doc.** Its id is the `id` field, else the file stem; its title is the first heading (the §4.2 title rule); its fields are the frontmatter keys other than `memex`; its text is the body after the frontmatter. Its label is `<name>:<id>`.
- **Entries: many docs in one file.** If a column-0 list item outside fenced code opens with a backticked id (the case-id grammar below), the file is an entry index: each such item is a doc, and the file itself is not one. This is how `CASES.md` works:

```markdown
---
memex: cases
---
# Namer cases

## Stability
- `NAME-001` Class names are stable across runs and platforms.
  Hash input is the canonical atom string, never object identity.
  Evidence: `tests/namer_stable.rs`
- `NAME-002` Two atoms with equal declarations share one class.
```

  - **Id.** `[A-Z][A-Z0-9]*(-[A-Z0-9]+)+`, for example `NAME-001`, `STT-WRAP-06` or `LIB-MEASURE-001`. An entry keeps its id forever; retired ids are not reused.
  - **Claim.** The rest of the first line, as one sentence stating what the entry proves. It is the entry's title.
  - **Detail.** Indented continuation lines (optional).
  - **Evidence.** Backticked paths inside the item that resolve to existing files (relative to the file's folder first, then the root). Optional, because an id in or on a test already makes `rg NAME-001` exact.
  - **Group.** The nearest heading above the item, recorded as the `group` field. The file's frontmatter keys are not copied to its entries.
  - **Planned entries.** An entry written before its test exists is fine, and is README-driven development applied to tests. Say so in the claim, for example "(planned)".
- **Duplicate ids.** Ids are unique within a collection; a duplicate is an `info` warning, `duplicate id <name>:<id> in <paths>` (each path once), and `ambiguous` in `show`; `search` lists one hit for the shared label. Keep them unique across the repo too, so a bare id names one doc (§3.1).
- **Existing structures.** Case-per-folder layouts and JSON case arrays need no conversion: a `folder` or `json` manifest (§4.2) indexes them, and their ids follow the same citation rules.

### 2.3 Citations and derived backlinks

memex scans every indexed doc (READMEs, headers, collection docs and entries, logs) for citations. Citations are never extracted from fenced code blocks (a fence is a line of three or more backticks or tildes indented at most three spaces; an unclosed fence runs to the end of the doc). A fence closes at a line of the same mark, at least as long, with nothing else on it; a backtick opener's info string holds no backticks. This one fence definition applies everywhere memex skips fenced code: citations, entries, titles and init's markers. Self-citations are dropped.

In a collection doc, the values of its frontmatter fields other than `memex` are scanned too, before its body, so metadata such as `case: NAME-001` or `spec: docs/spec.md` links both ways; each item of a list value is scanned on its own. A log's frontmatter is memex's own record of the run and is not scanned.

| Form | Matches |
|---|---|
| bare id `NAME-001` | any doc whose id matches the case-id grammar and is **declared** somewhere, so prose like `UTF-8` never links |
| `NAME-*` | every declared id with that prefix, so a hub README can cite a whole family |
| `collection:id` | that doc, when `collection` is a known name |
| path token | the file or folder it names (see below) |
| `[text](path)` | a file or folder in the walked set (no filesystem probes), resolved relative to the citing doc's folder, then the root. URLs and `#anchors` are not refs |

A path token is a maximal run of `[A-Za-z0-9._/@+-]` that contains `/` or a `.ext`, with trailing `.,:;)` stripped. It is resolved against the walked file set (no filesystem probes), relative to the citing doc's folder, then the root. It must stay inside the root, and a leading `/` means root-relative.

A prefix citation (`NAME-*`) is listed in **cited by** marked `via NAME-*`, but is excluded from roster `<-N`. That keeps `<-0` working as the orphan audit even when hub READMEs cite whole families.

`show` prints **cites** (each with its target's one-liner) and **cited by**. Explicit refs (markdown links that are not URLs or `#anchors`, and `coll:id` with a known collection) that resolve to nothing are listed as `unresolved`. Bare tokens never are.

## 3. CLI

```
memex search [<word|field:value>...] [--in <coll>[,<coll>...]] [--limit N] [--json]
memex show   <path|ref>... [--up N] [--comments] [--limit N] [--json]
memex log    <stream> [--to <id>[,...]] [--tag <t>[,...]] [--agent <id>] [--json]  <<'EOF' … EOF
memex info   [--agent <id>] [--json]
memex audit  [<path|glob>...] [--limit N] [--json]
memex init
memex --help | --version
```

- **Flags.** `--help` works after any verb and short-circuits: the arguments after it are not checked. `--` ends flag parsing. Unknown, repeated or misplaced flags are errors and are never swallowed. A flag that would have no effect is an error, judged on the verb's grammar, not on what the refs resolve to. `--json` anywhere before `--` makes usage errors JSON.
- **Bare `memex`.** Prints usage on stderr and exits 2.

### 3.1 Refs and labels

- **Labels round-trip.** Every printed label is valid input to `show`.
- **Intent docs** are labelled by root-relative path: `crates/namer/README.md`, `crates/namer/src/hash.rs`.
- **Other docs** are labelled `collection:id`: `cases:NAME-001`, `doom:2026-09-20-jettison`, `impl:20261013-214107-show-chain`.
- **Resolution order.**
  1. Absolute path.
  2. Relative path: tried against both the cwd and the root. "Exists" means present in the walked set, matched case-sensitively, so `readme.md` never finds `README.md` on a case-insensitive filesystem. If exactly one exists, it wins. If both exist and differ, the result is `ambiguous`. `.` is the exception: it is the root's own label, so it always means the root.
  3. `collection:id`, exact.
  4. A bare id that is unique across collections, matched exactly, then case-insensitively.
  5. A unique basename.
  6. If several still match, memex lists candidates and exits 1. It never guesses.
- **Other worktrees.** A path outside the current root resolves against *its own* nearest root (a `.memex/` or toplevel (§4.1) above it). With neither, its root is the path itself for a folder, or its parent for a file; memex never walks the enclosing filesystem. `memex show ../wt-cook-3/src/x.rs` reads that worktree's intent layer, so no `@worktree` suffix is needed. Its `cites` and `cited by` would name labels of that root, which don't resolve from the cwd, so text prints `(in another root: run show from <root>)` in place of each list and JSON gives `cites: null` and `cited_by: null`; nothing is dropped silently. Logs are shared anyway.
- **Built-in names.** `readmes` and `headers` are the graph; `logs` means every stream. A collection may share one of these names, with a warning (§4.2 Names).

### 3.2 `search`

- **No arguments.** Usage error (exit 2) pointing to `memex info`, which is the one overview.
- **Words.** Ranked BM25 with field weights, English stemming and prefix matching. Fuzzy matching allows one edit for words of 5 or more characters, and those hits are marked `[~variant]`. A word is a run of Unicode letters and digits, lowercased, so `naïve`, `café`, `Straße` and CJK runs are words. Word lengths and edits count characters (Unicode scalar values), not bytes, so `naïve` is one edit from `naive` (a word under 5 characters, like `café`, gets prefix matches but no fuzzy ones). Ids and paths are also kept as whole tokens, so `NAME-001` matches exactly; a whole token also matches the longer ids and paths it begins where the next character is `-`, `/` or `.`, as an exact match, so `ATM-NAME` finds `ATM-NAME-01` and `src/index` finds `src/index/mod.rs`, while `NAME-1` never finds `NAME-10`. Any word may match unless an operator says otherwise; more matches rank higher. Scores are only comparable within one group. Ranking compares scores quantized to `1e-9`, then by ref, so libm differences across OSes cannot reorder results.
- **Operators `+` and `!`.** A term's first character may be an operator; the rest of the term is read as usual, as a filter or as words, and a term of several words applies the operator to each.
  - `+word` keeps only the docs that hold the word, and ranks as `word` does.
  - `!word` leaves out the docs that hold the word, and never ranks. A query whose only words are `!word`s lists, or with `--in` rosters, the docs that remain.
  - A doc *holds* a word when the text the index reads has it or a stem-mate, or, for an id or path, that token or a longer one it begins (above). Prefix and one-edit matches never count, so `+namer` leaves out a doc that only says `names`, and `!test` keeps a doc that says `testament`.
  - `!field:value` keeps exactly the docs that the filter would drop, docs without the field among them, so `!since:2d` keeps docs that have no `time`. `+field:value` is the filter.
  - Operators never change a score: they decide which hits stay, and `showing N of M` counts those. A word given bare and with `+` is required; a word both required and left out matches nothing.
  - `-word` is not an operator: before `--` it is an unknown flag, and `search`'s error names `!word`.
  - **Quoting.** The non-interactive shells that agents run pass `!` as is. At an interactive bash or zsh prompt `!word` is history expansion: single-quote it, `'!word'` (double quotes do not stop it in bash). `+` needs no quotes.
- **Filters `field:value`.** Exact and case-insensitive, combined with AND. Reserved: `agent:`, `to:`, `tag:`, `branch:`, `worktree:` (basename or path), and `since:` (`30m`, `6h`, `2d`, `1w` or `2026-10-13`; it keeps only docs that have a `time`). Any other `x:` is a filter only if some doc in scope has field `x`; otherwise it is a literal word. That keeps `std::io` and refs searchable.
- **Markers.** A group heading names the markers its lines use: a roster group's reads `cases  (showing 50 of 214; <-N cited by N)`, and a group with a fuzzy hit adds `; [~w] fuzzy match on w`. JSON is unchanged.
- **Grouped output, always.** Each collection with hits is one group. The fixed order is the graph (`readmes`, `headers`), then collections alphabetically, then `logs` as one group. Empty groups are omitted.
  - **Limits.** Without `--in`, `--limit` applies per group (default 5). With `--in`, the default is 20.
  - **Why groups.** Grouping is why READMEs and headers can be in global search: thousands of headers cannot drown a README, and log noise stays in its own group.
- **Zero hits.** stdout stays empty and the exit is 0, so scripts are unaffected; stderr prints `memex: no hits for <words>; try fewer or other words, or memex info`, naming the terms that are not filters as typed, operators included (with filters only: `memex: no docs match <filters>; try fewer filters, or memex info`; with a single filter it reads `memex: no docs match <filter>; try another value, or memex info`), in text and JSON modes.
- **Filters without words.** Lists matching docs, newest first for logs.
- **`--in X` with no words.** Prints the roster: every doc's one-liner, logs newest first, everything else by id. Each roster line carries `<-N`, its cited-by count (prefix citations excluded), so `<-0` marks docs that nothing cites. Default limit 50.
- **`--in` names.** `readmes`, `headers`, `logs`, collection names and stream names. A name a collection shares with the graph or a stream means the graph or the stream (§4.2 Names). An unknown name is a usage error (exit 2), never a silently empty result.

```
readmes  (showing 2 of 2)
  crates/namer/README.md  Namer: turns atoms into stable class names
headers  (showing 5 of 31)
  crates/namer/src/hash.rs  FNV-1a over canonical atom strings  [~hashes]
cases  (showing 1 of 1)
  cases:NAME-001  Class names are stable across runs and platforms.
logs  (showing 1 of 1)
  impl:20261013-214107-show-chain  2h · cook-2 · feat/show  show walks hub READMEs
```

### 3.3 `show`

It takes one or more refs and prints a section for each, separated by `=== <label> ===` lines. A ref repeated in one call prints once, at its first position. A requested file that can't be read, or a requested folder that can't be listed, is that ref's error (`io`), so `show` exits 1 as for any failed ref; unreadable files met while loading the corpus stay warnings (§4.2). A path inside a folder that can't be listed was never walked, so it is `not found`; the folder's own warning says why.

- **File.** The header (full content for `.md`, after any frontmatter: see Frontmatter below), then `cites`, then `cited by`, then the chain. `--up N` limits the depth; `--up 0` prints the file alone.
- **Folder.** The folder's README if there is one; otherwise `(no README)`. Then the children (the folder's own README.md excluded, since it is already printed): subfolders with their README one-liners, and files with their header one-liners or `(no header)`. A Markdown file has no header (§2.1), so its child line shows its title instead: the §4.2 title rule applied to its text after any frontmatter (first ATX heading outside fenced code, else first non-blank line). The title is looked for in the file's first 8 KiB (§2.1), whole lines only. An empty Markdown file, or one that can't be read (its warning stands), shows its name alone, and its JSON `title` is empty. No placeholder, so no legend line. JSON `title` carries the same text. Then `cites`/`cited by` for the folder path, then the chain. A folder without a README lists its children and is not an error. `show .` is the map of the root.
- **Comments.** `show` prefers the header. With `--comments`, a file section also prints the file's other comment blocks after the header, under `comments  (showing N of M)`, in file order, each headed `L<n>`, its first line number. A comment block is a run of lines that start, after indentation, with the file's line-comment syntax (§2.1), or one block comment; a comment after code on the same line is not read. For this, `show` reads the whole file, up to 1 MiB; a file cut there ends the list with `… (comments cut at 1 MiB)`. The list uses `--limit` (default 20). A file with no header prints its first 3 comment blocks this way without the flag, its heading adds `--comments for all` when there are more, and the `(no header)` gap line (below) stands as the warning. Comments are printed, never indexed: `search` does not read them and they cite nothing. On a folder, an entry or a log, `--comments` is ignored. The list sits after the body and its gap line, before `cites`, with `--up 0` too. Each row is `  L<n>`, then the block's lines with the markers removed, as the header prints them, indented four spaces. A block runs as the header does (§2.1): blank lines inside it do not end it, and it ends at the first line that is neither comment nor blank. The blocks listed are those after the header, or after the lines §2.1 skips (shebang, pragmas, the license block) when there is no header; a docstring other than the module docstring is not a comment. A list cut by `--limit` has the heading `comments  (showing N of M; --limit M for all)`; the no-header list shows 3, or `--limit` if lower, and is left out when the file has no other comment block; with `--comments`, an empty list prints `(none)`. A file with no comment family (Markdown, an unknown extension) or a binary file has no comments: `--comments` is ignored there and nothing falls back. JSON `comments` is present exactly when the text prints the list; `line` counts from 1 and `text` is the block as printed, unindented. The no-header heading says `--comments for all` even when `--limit` is what cut it; a file cut at 1 MiB counts M over the blocks read, and JSON `cut` says the file was cut there; a blank line inside a block prints empty; a Python docstring that stands before any statement, after a `#` header, is the module docstring and is listed. Any other triple-quoted string is skipped whole. A read cut at 1 MiB drops its unfinished last line; with `--comments` and no block read, `(none)` comes before the cut row. Pragma-like lines after the header that the family reads as comments (`//go:`, `{-#`) are listed.
- **Chain.** Ancestor READMEs in full, nearest first, root last. For a README.md itself, the chain starts at the folder above its own, since the file is already printed in full.
- **Frontmatter.** Text output prints a README or a Markdown file shown by path (a folder's body, a `.md` file's body, each chain README) from after its closed frontmatter block (§2.2 rule), with the blank lines after the block dropped; a file with no closed block prints whole. The block is metadata, such as a root mark (§4.2 Roots), not intent, and no metadata line replaces it: a collection doc's fields print under `fields` when it is shown by `<name>:<id>`. JSON `text` (folder, file and chain) keeps the whole file, frontmatter included (2026-10-20).
- **Entry** (§2.2). Claim, detail, group, the defining file, evidence paths, and cited by.
- **Log or other collection doc.** The full text, its fields as a `key: value` block, cites, and cited by.
- **Limits.** The children and cited-by lists use `--limit` (default 50) and always print `showing N of M`, adding `(--limit M for all)` only when the list was cut (N < M).
- **Gaps.** A section that prints `(no README)` or `(no header)`, as its body or on a child line, follows its children (a file section: its body) with one line for each placeholder it used, once: `(no README): add README.md here; line 1 is # Name, then a paragraph saying what the folder is` and `(no header): add a comment block atop the file; its first sentence states the file's purpose`. JSON is unchanged.
- **Exit code.** If any ref fails, memex still prints the others and exits 1.

### 3.4 `log`

- **Body.** Read from stdin, required and non-empty. If stdin is a TTY, memex exits 2. The maximum is 16 KiB, measured on the body as stored (after the BOM is dropped, CRLF becomes LF and the bytes are decoded), which exits 2 with the message "logs are findings: cite artifacts by path". The title is the first non-blank body line after any frontmatter, with leading `#`s and spaces stripped. If the title is empty, memex exits 2. The slug is made from the title: lowercase ASCII alphanumerics, other runs become `-`, trimmed, at most 60 chars, or `entry` if empty.
- **Store.** `log` is allowed if the root or the main checkout has `.memex/`. With neither, it exits 3 with "run `memex init`" (§4.1).
- **Extra fields.** An optional leading `---` frontmatter block in the body is merged in. Collisions with memex-owned keys exit 2. Keys compare case-insensitively, both against memex's keys and against each other, so `To:` cannot forge a `to` field. A value containing a lone carriage return exits 2. The block ends at its first `---` line, as in YAML, so a value cannot hold one (accepted limit).
- **Stream name.** `[a-z0-9][a-z0-9-]{0,39}`. It must not be `readmes`, `headers` or `logs`, a Windows device name (`con`, `prn`, `aux`, `nul`, `com0`–`com9`, `lpt0`–`lpt9`; refused on every OS so a store stays portable), or an existing manifest name (matched against the listed file names in `.memex/collections/`, exactly, never by probing). The manifest-name check is unchanged (listed files in `.memex/collections/`); a frontmatter collection never blocks `log`, it only draws the shared-name warning on reads. Streams need no declaration.
- **Agent id.** Taken from `--agent`, then `$MEMEX_AGENT`; otherwise it is omitted. No harness exposes a per-agent id in the environment (Spike 2), so there is no third source.
- **Output.** stdout is exactly the ref on one line, so `ref=$(memex log impl <<'EOF' … EOF)` works. With `--json`, the output is `{"ref","path"}`.
- **Immutable.** There is no edit verb. A correction is a new entry that cites the old ref, and the old entry then shows it under "cited by".

### 3.5 `info` (the atlas)

Where things are, never their contents. It prints no log bodies or titles, only names, counts and the commands to get there. It's the overview for humans and agents, and what the start hook prints. Text output is capped at **3,000 bytes**; long lists end with `+N more`.

```
memex  /repo · log store .memex/logs/
graph        48 readmes · 912 headers
collections  cases 214 · doom 41 · neo-cases 294 · perf 212
streams      impl 12 (3 new in 24h) · review 3 (1 new) · drift 5
to cook-2    2 messages → memex search to:cook-2
here         crates/namer — Namer: turns atoms into stable class names
use          memex show <path|ref> · memex search <words> · memex log <stream> <<'EOF' … EOF
```

- **Header.** The root, then the log store: root-relative when the store is inside the root, else absolute.
- **Collections.** Name and doc count, alphabetical, from frontmatter and manifests alike; `--json` adds kind, description and owner. Invalid manifests, invalid collection names, duplicate ids and shared names (§4.2) are listed as warnings.
- **Streams.** Total entries and how many are new in the last 24 hours, ordered by most recent activity.
- **`to` line.** A count only: every entry in the store addressed to the id. Shown when an agent id is known (`--agent`, `$MEMEX_AGENT`), including at zero (`to cook-2    0 messages → memex search to:cook-2`), so an agent can tell that it is wired up; JSON `to` is then always present.
- **`here` line.** Shown only when the cwd is not the root: the nearest README's one-liner.

### 3.5b `init`

The one-step setup, idempotent. Inside a repo (a toplevel, §4.1) it targets the toplevel; outside one, the root that discovery finds (§4.1 item 1: the nearest folder above with `.memex/`), else the cwd, so init never creates a second store that shadows one above it. When that root is above the cwd, init's report opens with the line `using the store at <root>; to start a new one here instead: mkdir .memex && memex init`, where `<root>` is the absolute, `/`-separated path; inside git, or when the target is the cwd, the line is not printed. A `.memex/` in `$HOME` is adopted like any other, and named the same way. It reports each action.

1. Creates `.memex/` (never overwrites existing files):
   ```
   .memex/
     .gitignore          # logs/ and cache/, ignored by default
     config.json         # {"max_logs_mb": 10}
     collections/        # manifests go here (empty but for a .gitkeep, so git keeps it)
   ```
2. Writes the AGENTS.md snippet (§6) into `AGENTS.md` between `<!-- memex:begin -->` / `<!-- memex:end -->` markers, creating the file if missing and replacing the block on re-run. If `CLAUDE.md` exists without `@AGENTS.md`, it appends `@AGENTS.md` as a final line. CLAUDE.md is never created. Markers and the `@AGENTS.md` line count only outside fenced code blocks. memex never writes through a symlinked `AGENTS.md` or `CLAUDE.md`: it warns and leaves them alone. If `AGENTS.md` has no markers but already holds the snippet's heading line (`## memex: read intent before code`) outside fenced code, init leaves the file alone and warns `memex: warning: AGENTS.md already has the memex snippet without markers; wrap it in <!-- memex:begin --> / <!-- memex:end --> to let init manage it`.
3. If no `memex` executable is on PATH (e.g. run via `npx`; PATH entries inside a `node_modules/.bin` folder are ignored, since that is npx's temporary shim, while a global npm install counts, and on Windows `memex.exe` or `memex.cmd` count), copies its own binary to `~/.local/bin` (`%LOCALAPPDATA%\Programs\memex` on Windows) and prints the plain line `add to PATH: <dir>` if that folder isn't on PATH. The install dir comes from the `$HOME` / `%LOCALAPPDATA%` env vars, not the known-folder API, so tests can redirect it. An installed binary is replaced only if its bytes differ.
4. Prints `memex info` for the target, without an agent id (init has no `--agent`, so its output depends only on the repo).

If another process creates one of init's files between init's check and its write, init leaves that file untouched and exits 3 with `memex: cannot create <path>: another process created it while init ran, or a name that differs only in case holds it`.

Reads work without `init` (the intent layer needs zero config); `log` needs a `.memex/`.

### 3.5c Messages

Agents are memex's main readers, so every error that leaves the user stuck names the next step, in the same line:

- `not found: <arg> (find refs with memex search <words>)`
- `unknown collection '<name>'; collections: <names, comma-separated, in §3.2 group order>`
- `log body is empty: memex log <stream> <<'EOF' … EOF`
- `unexpected argument '<word>'` for `log` adds `: log reads its body from stdin: memex log <stream> <<'EOF' … EOF`
- `unknown verb '<word>'` adds `; run memex --help`
- `log needs a stream name` adds `: memex log <stream> <<'EOF' … EOF`
- `stream name '<name>' is reserved` adds `; pick another name, such as impl`

Exit codes and JSON codes are unchanged.

### 3.5d `audit`

How healthy the intent layer is, for the whole root or part of it: where intent is missing or thin, which files have grown too long, and which references are broken. It is a report of facts memex already knows, plus one plain-text measure, line count. It never parses code (§1, §7).

- **Scope.** With no argument it audits the root. Each argument is a path, resolved as `show` resolves one (§3.1 steps 1, 2 and 5; a label or a bare id is not a scope and is `not found`), or, if it holds `*`, `?` or `[`, a glob matched against walked root-relative paths with the gitignore rules that manifest `include` uses (§4.2). A file is in scope if it is named or lies under a named folder or matches a glob; a folder likewise. A path that names nothing is that argument's `not found` error (§3.5c), the rest are still audited, and the exit code is 1; a glob that matches nothing is not an error.
- **Kinds.** Each finding has one kind. The kinds print in this order, and a kind with no finding in scope is left out:
  1. `headers`: a file with a comment family (§2.1) that is not empty, not binary (§2.1 skips binary files, so `length` and `thin` leave them out too) and has no header.
  2. `readmes`: a walked folder with no README.md whose subtree holds a file with a comment family (§2.1), the root included. A folder of data alone (case folders, JSON or text fixtures) is not a unit and needs none.
  3. `length`: a file with a comment family whose code lines exceed 500. Code lines are its lines minus its header block and its license block (§2.1); blank lines and other comments count. Detail: `<N> code lines`.
  4. `one-liners`: a header or README whose one-liner (§2) is over 100 characters (Unicode scalar values). Detail: `<N> characters`.
  5. `thin`: files of 80 lines or more that have a header, ordered by header lines over total lines, lowest first (ties by path). Detail: `<H>/<T> header lines`. It points at where intent is thin; it has no threshold, so it is never "fixed", and it is not a finding: the first line's count, `nothing to fix` and JSON `findings` leave it out.
  6. `cites`: an explicit reference in a README, header or collection doc that resolves to nothing (`unresolved` in `show`, §2.3). Detail: `→ <reference as written>`. Logs are scaffolding and are not audited.
  7. `ids`: a duplicate id within a collection (§2.2). The finding is `<name>:<id>`; detail: its paths, each once.
  8. `roots`: a collection root that cannot work as written (§4.2 Roots). A README whose `memex:` key names no manifest: the README's path, detail `→ <name>: no manifest`. A manifest with no root, explicit or marked: its file, `.memex/collections/<name>.json`, detail `no roots`; it is in scope only when the root is. A root inside another root of the same manifest: the inner folder, detail `<name>: inside <outer>` (`.` for the root). Fix: `(roots): give each marked collection a manifest and each manifest a root, and nest no root inside another`.
  A manifest doc is in scope by its folder (§4.2). Prefix citations, uncited docs and anything about code bodies are never findings.
- **Lines.** A line ends at LF; CRLF counts once; a last line without a newline counts if it is not empty. `length` and `thin` count a file's whole text, so `audit` is the one verb that reads past the 8 KiB head (§2.1), and only for files with a comment family; the header and license blocks are found in the head as §2.1 finds them. `info`, `search` and `show` never read further.
- **Output.**

```
audit       src · 148 files · 12 folders · 7 findings
headers     (showing 2 of 2; files with no header)
  src/a.rs
  src/b.rs
(no header): add a comment block atop the file; its first sentence states the file's purpose
readmes     (showing 1 of 1; folders with no README)
  src/util
(no README): add README.md here; line 1 is # Name, then a paragraph saying what the folder is
length      (showing 1 of 1; over 500 code lines)
  src/big.rs  612 code lines
(length): split the file; give the part with its own purpose its own file and header
cites       (showing 1 of 1; unresolved)
  src/z.rs  → NAME-404
(cites): fix the reference or remove it
ids         (showing 1 of 1; duplicate)
  cases:NAME-003  tests/a/CASES.md · tests/b/CASES.md
(ids): give each doc its own id
```

  The first line names the scope (`.` for the root; several arguments are joined with ` · `), the walked files and folders in it, and the total findings. Each kind is a group headed like a search group, `<kind>  (showing N of M; <what it lists>)`, its lines are `<label>  <detail>`, and its fix line follows; `one-liners` says `(one-liners): shorten the first sentence to 100 characters or fewer`, and `thin` has no fix line. With no finding, the first line is followed by `nothing to fix`. Each kind's list uses `--limit` (default 20); when it is cut, the heading reads `(showing N of M; --limit M for all; <what it lists>)`.
- **Exit code.** 0 whatever it finds, since a finding is not an error; 1 if a path argument names nothing; 2 for a usage error.

### 3.6 JSON

With `--json`, stdout is always one JSON document, including errors. Without it, errors go to stderr as text.

- **Doc:** `{ref, collection, id, path, title, fields:{…}}`. `fields` holds frontmatter and manifest fields (an entry's are `group` and `evidence`); logs add `time, agent, to[], tags[], branch, worktree, commit`.
- **`search` (with words or filters, or a roster):** `{query, filters:{…}, groups:[{collection, total, hits:[doc + {score|null, snippet, fuzzy:[]}]}]}`. Roster hits have `score: null` and `cited_by: N`, which, like `<-N`, excludes prefix citations. JSON scores are rounded to 3 decimals. The shape is the same with or without `--in`. A filter left out by `!` is listed under its name with the `!`, as `"!tag": ["show"]`; `+` adds nothing to the name.
- **`info`:** `{root, log_store, collections:[{name, kind, builtin, count, description, owner, new_24h?, newest?, error?}], to?:{agent, count}, here?:{path, title}, warnings:[]}`, where `kind` is `null` for `readmes` and `headers` (`builtin: true`), `markdown` for a collection declared by frontmatter, the manifest's kind (`folder`, `json`) for one declared by a manifest, and `log` for streams.
- **`show`:** `{docs:[doc + {text, cites:[{ref, title, resolved}], cited_by:{total, refs:[{ref, title, via?}]}, children?:{total, items:[{path, kind:"dir"|"file", title}]}, comments?:{total, cut, items:[{line, text}]}, chain:[{path, text}]}], errors:[{arg, code, message, candidates?}]}`. `cites` and `cited_by` are `null` for an entry in another root (§3.1).
- **`audit`:** `{scope:[…], files, folders, findings, kinds:[{kind, total, items:[{ref, detail}]}], errors:[{arg, code, message}]}`, where `kinds` holds only kinds with items, in §3.5d order, each list bounded by `--limit` (`thin` is listed there but, as a pointer, is not counted in `findings`); `detail` is `null` where the text has none.
- **`log`:** `{ref, path}`, where `path` is absolute and `/`-separated.
- **Fatal errors:** `{error:{code, message}}`, where `code` is one of `usage | not_found | ambiguous | no_root | io`. `no_root` is `log` with no `.memex/` (§4.1).
- **Exit codes:**
  - 0: ok, including zero hits.
  - 1: not found or ambiguous.
  - 2: usage error.
  - 3: environment error: no `.memex/` for `log` (§4.1), store not writable, or an I/O failure that stops the verb.
- **Closed stdout.** If the reader closes stdout early (`memex search … | head -1`), memex stops writing, prints nothing on stderr and exits with the code it would otherwise have (0 on success); a closed stdout is never itself an error. A `show` ref that fails on I/O is that ref's error and exits 1 (§3.3).
- **Determinism.** Lists are sorted by byte (code point) order of their labels, never by locale (ties broken by ref) unless a rule gives another order: cites and evidence keep the order they are written in. `$MEMEX_NOW` is RFC 3339 UTC with `Z`, and it replaces the clock everywhere: log time, prune, `since:`, ages and `new_24h`. An invalid value exits 3. That lets fixtures compare stdout byte for byte.
- **Ages.** Ages print as `Nm` below 60 minutes (`<60m`), `Nh` below 48 hours (`<48h`), else `Nd`.

## 4. Storage

### 4.1 Discovery

1. **Root.** memex walks up from the cwd, or from the path argument for `show`, to the nearest folder that contains `.memex/`; the nearest one wins, as with git. The walk stops at the **toplevel**: the nearest folder at or above the start whose listing has an entry named `.git`, of any kind (a folder in a checkout, a file in a linked worktree or submodule), so a `.memex/` above a repo (say in `$HOME`) never captures it. If there is none, the root is the toplevel, or the start folder when no `.git` lies above it. The toplevel is found by listing, never by asking git, so reads and `init` behave the same whether git is present, missing or failing. Accepted limits: git's environment and config are not consulted (`GIT_DIR`, `GIT_WORK_TREE`, `core.worktree`, `GIT_CEILING_DIRECTORIES`, `safe.directory`, filesystem boundaries); a `.git` entry git would reject (an empty folder, a `.git` file naming a missing git dir) still marks a toplevel; a cwd inside `.git` belongs to the repo that holds it. `log` then needs `.memex/` in the root or the main checkout (item 3).
2. **Intent layer.** It is read from the root's tree, live, using the `ignore` walker. That means `.gitignore` and `.ignore` are honoured (`.ignore` is the way to exclude paths from memex and rg together). The built-in skip list is hidden entries, anything git-ignored or `.ignore`d, `node_modules` and `target`. A manifest's explicit roots may point into hidden folders; the roots READMEs mark are found by this walk (§4.2 Roots). A README is matched by its listed filename, `README.md`, exactly. memex never probes with `exists()`, so behaviour is portable across case-insensitive filesystems.
3. **Log store.** `.memex/logs/`, ignored by default (`.memex/.gitignore`). Inside a git worktree it resolves to the **main checkout's** `.memex/logs/` (the same root path in the main worktree, found via `git rev-parse --git-common-dir`, or without git from the layout, §4.4 git), so every worktree shares one store and logs survive `git worktree remove`. A bare repo has no main checkout, so its store is `<git-common-dir>/memex/logs/`; there `log` needs `.memex/` in the root only, and no `.gitignore` is written because the store is inside the git dir. `log` is allowed if the root or the main checkout has `.memex/`. With neither, it exits 3 with "run `memex init`". Creating the store also writes `.memex/.gitignore` when it's missing, so logs can never be committed by accident. memex creates the folder on the first `log`.
4. **`.memex/` contents.** Committed: `collections/<name>.json` manifests, `config.json`, `.gitignore`. Ignored by default: `logs/` and `cache/`. A team that wants to keep logs removes `logs/` from the `.gitignore`; that's their choice. `config.json` has two keys, both optional: `max_logs_mb` (the size cap, default `10`) and `keep_logs` (an age limit, off unless set). It is read from the store owner's `.memex/config.json` (the main checkout's, for a worktree); a missing file means the defaults. Any other key makes the file unusable (§4.3 Cleanup).

### 4.2 Manifests

A manifest indexes a structure that is not Markdown: one doc per folder that holds a marker file, or per entry of a JSON array. Markdown needs no manifest (§2.2).

```json
{
  "name": "rs-cases",
  "description": "reference-rs cases: one doc per case folder (case.json + README + spec header).",
  "owner": "agent-rs skill",
  "kind": "folder",
  "roots": ["packages/reference-rs/modules"],
  "marker": "case.json",
  "merge": ["README.md", "spec.ts"],
  "id": "id",
  "title": "name",
  "weights": { "id": 4, "title": 3 }
}
```

| Key | Kinds | Meaning |
|---|---|---|
| `name` | all | label prefix; must equal the file stem; `readmes`, `headers` and `logs` rejected |
| `description`, `owner` | all | shown in `info`; `owner` names the skill that builds it |
| `roots` | all | root-relative folders (optional; unioned with the folders that READMEs mark, below) |
| `include` | json | globs under roots |
| `id` | all | folder: marker field or folder name (default); json: entry field (required) |
| `title` | all | field for the one-liner (default, for every kind: the first ATX heading outside fenced code, 1–6 `#` then a space or tab, else the first non-blank line; an empty title field falls back to the default; no first-sentence cut) |
| `weights` | all | per-field ranking boost (defaults: `id` 4, `title` 3, body 1) |
| `marker`, `merge` | folder | marker file defines a doc; `merge` globs in that folder are appended to the body |
| `at` | json | JSON Pointer to the entries array (default: document root) |

- **Roots.** A manifest reads its `roots`, in the order given, then every walked folder whose README.md has `memex: <name>` in its frontmatter, `<name>` being the manifest's name, in walk order; a folder named both ways is read once. So a collection grows by adding a README, with no edit to its manifest:

```markdown
---
memex: lib-cases
---
# Slider cases
```

  A marked folder is found by the intent walk (§4.1 item 2), so it honours the same ignores: `.gitignore`, `.ignore`, hidden entries, `node_modules` and `target`. An explicit root may still point into a hidden folder. Marking reads nothing extra: the walk already reads every README whole. The README stays the graph's doc and is not a collection doc, unless the manifest's `marker` or `merge` names it. A manifest with no root reads nothing, and roots that nest are each read, so the inner one's docs come twice; `audit` lists both (§3.5d kind 8).
- **`folder`.** Each folder containing `marker` is one doc, and that folder is the doc's folder for its relative citations (§2.3). The marker's fields come from top-level JSON scalars, or from frontmatter for a `.md` marker. Each `merge` file adds its full text if it is Markdown, or **only its header** if it is code, so memex still never reads code bodies.
- **`json`.** Each entry of the array at `at`, in every matched file, is one doc. Scalars become fields and strings are indexed as text. The JSON file's folder is the doc's folder for its relative citations (§2.3).
- **Frontmatter.** Flat YAML: `key: scalar`, `key: [a, b]`, or `- item` lists. Anything nested is kept as text and cannot be filtered. Frontmatter is optional, and missing fields simply don't score.
- **`kind: markdown`.** No longer a kind: the manifest is skipped with `memex: warning: <path>: kind 'markdown' is gone; add 'memex: <name>' to its files' frontmatter instead`.
- **One collection, two sources.** A manifest and frontmatter files that name the same collection form one collection; `info` shows the manifest's kind, description and owner.
- **Names.** A frontmatter collection named like the graph (`readmes`, `headers`), like `logs`, or like a log stream still loads (a manifest named `readmes`, `headers` or `logs` is rejected, as the table says; one named like a stream loads): its docs are indexed, appear in global search under their own group, and resolve in `show` as `<name>:<id>`. `--in <name>` keeps meaning the graph or the stream. A warning prints once per run in the form above, `memex: warning: collection '<name>' (in <path>) shares a name with the graph; --in <name> means the graph — rename it to reach it with --in` (for `logs` or a stream: `shares a name with a log stream; --in <name> means the stream`), where `<path>` is its first file in byte order (for a manifest's docs, the manifest file), and `info` lists it. It prints on every verb that loads the corpus.
- **Unreadable files.** A README, header, Markdown file or stream folder that can't be read is a warning (`memex: warning: cannot read <path> (<ErrorKind>)`) and is skipped. A path whose name memex skips (§4.4 Encoding) gets no warning either, nor does a walked file with no comment family (§2.1) that is not Markdown: the walk never opens it. A folder manifest's `merge` file is read by its manifest whatever its family, and warns as above; warnings print in the order they arise. A folder manifest's `merge` file that can't be read gets the same warning. A manifest root that is a symlink resolving outside the repo root is skipped with `memex: warning: <collection>: root <root> leads outside the repo; skipped`. It never fails a read verb.
- **Invalid manifests.** An invalid manifest or an unknown key is a warning, and that collection is skipped. It never makes the run fail. Warnings are printed once per run on stderr by every verb that loads them, in text and JSON modes. `info` also lists them, with duplicate ids, in its output.
- **Duplicate ids.** Within one collection, reported as `info` warnings, and as `ambiguous` in `show` (§2.2).

### 4.3 Log files

Each entry is one file, `<store>/<stream>/<YYYYMMDD-HHMMSS>-<slug>.md` in UTC, and the id is the filename without `.md`. The slug is derived from the title (§3.4). memex writes the frontmatter. The body is stored as given, after any frontmatter the agent supplied, normalised only as §4.4 Text says (no BOM, LF line endings).

```markdown
---
stream: impl
time: 2026-10-13T21:41:07Z
agent: cook-2
to: [orchestrator]
tags: [namer, show]
branch: feat/show
worktree: /Users/ryn/Developer/memex-wt/cook-2
commit: 3f2a9c1
---
show walks hub READMEs
`show <file>` now lists "cited by"; crates/README.md cites NAME-* …
```

- **`commit`** is HEAD at write time. It lets a reader judge whether a finding has gone stale.
- **Concurrency.** There are no locks.
  - **Write.** memex writes a dot-prefixed temp file in the stream folder, then persists it to the final name without overwriting (`persist_noclobber`, atomic on all platforms). If the name exists, it retries with `-2`, `-3`, ….
  - **Read.** Readers ignore dotfiles, so a partial entry is never visible.
  - **Vanished entries.** An entry or stream folder that disappears between listing and use (another process pruned it) is already gone: prune and readers skip it silently (`NotFound` never warns, nor does removing a stream folder a writer has just landed in; every other error kind still does). A write whose stream folder is removed under it re-creates the folder and retries, up to 64 attempts (each concurrent prune can remove the folder at most once per run, so a writer loses at most one race per pruner).
  - **Accepted limit.** A stream folder swapped for a symlink between memex's check and its write is followed; closing that race needs `openat`, which std does not offer portably. The store is trusted like the repo it lives in.
- **Cleanup.** Logs are ephemeral, like chats: useful in the moment, stale after a while. The store is a size-capped rolling window, pruned on every `memex log` after the new entry lands. There's no background process; pruning happens only on write, and reads never delete. Anything worth keeping belongs in a README, a header or a collection, not a log. `rm -rf` still works.
  - **What counts.** Only regular files named `YYYYMMDD-HHMMSS-<slug>.md` (a real UTC stamp, a slug of at least one character) directly inside a stream folder are entries: they alone are counted and they alone are ever deleted. Hidden names, symlinks, subfolders and stream folders that are symlinks are neither counted nor touched. The cap covers the whole store, every stream together (the main checkout's store, which worktrees share). An entry's size is its file length in bytes.
  - **Size cap.** `max_logs_mb` is a positive whole number of MiB (1 MiB = 1,048,576 bytes), default `10`. After the write, if the entries total more than the cap, memex deletes the oldest until the total is at or under it. Oldest is by filename stamp; ties are broken by entry name, then stream name, in byte order. The entry just written is never deleted, though its size counts. A delete the OS refuses warns, its bytes still count, and the next oldest goes instead; an entry that vanished counts as gone.
  - **Age limit.** `keep_logs` is optional and off unless set. Set, it deletes every entry older than it, by filename stamp, before the size cap is applied; when both are set, both apply. It takes `Nm`, `Nh`, `Nd`, `Nw` or `forever` (no age limit). A far-future `$MEMEX_NOW` makes every entry stale, so a `log` under it with `keep_logs` set prunes the whole store but the new entry; `$MEMEX_NOW` is a fixture clock, not a setting (accepted).
  - **Bad config.** memex never deletes on config it can't read. A config file that cannot be read, is not valid JSON, is not a JSON object, or has an unknown key prints one warning on stderr and nothing is pruned at all. A value outside its grammar prints one warning that names the key and the value, and disables only its own limit: `invalid max_logs_mb <value>: not pruning by size` (0, a negative, a fraction, a string, or a cap too large to count in bytes), `invalid keep_logs <value>: not pruning by age`; the other limit still applies. A string value is shown in single quotes, any other value as its JSON text.
  - **Folders.** Stream folders emptied by a prune are removed; a folder that was already empty is left alone.

### 4.4 Portability

One codebase for Linux, macOS and Windows, using only cross-platform APIs. No manual Windows testing; **CI runs the fixture suite on `ubuntu`, `macos` and `windows`**.

- **File modes.** Files memex creates (log entries, init's files) get the ordinary mode for new files (0644 before umask on unix), not tempfile's private 0600; a file memex replaces keeps its mode.
- **Atomic writes.** `tempfile` + `persist_noclobber` (atomic, never overwrites) on every platform, in place of raw `link()`. If persisting fails and the target name exists in any form (Windows reports a directory there differently), the name counts as taken and the `-2`, `-3` retry runs. Deletes that the OS refuses (on Windows, a file held open by another process) are warnings, never errors.
- **Paths.** Labels and JSON always use `/`. Canonicalise with `dunce` (no `\\?\` prefixes). Compare paths case-sensitively in labels, since filesystem case behaviour varies.
- **Filenames.** Log names have no colons. Slugs are `[a-z0-9-]`, max 60 chars. Windows reserved names are moot for entry files, because the timestamp prefix comes first; stream folders refuse them (§3.4).
- **Name matching.** Special names (`README.md`, manifest files, `.memex` and everything memex reads inside it: `config.json`, `collections/`, `logs/`) are matched against directory listings, exactly. The `.md` extension is matched on the listed name, exactly (`NOTES.MD` is not Markdown). memex never decides by `exists()`, so case-insensitive filesystems behave like Linux.
- **Text.** Accept `\r\n` everywhere (headers, Markdown docs, frontmatter, log bodies). A leading UTF-8 BOM is dropped on read, so frontmatter still parses from shells that add one. Write `\n`, without a BOM. The exception is a user file memex edits in place (`AGENTS.md`, `CLAUDE.md`): it keeps that file's line endings.
- **Encoding.** Non-UTF-8 file content is decoded lossily; non-UTF-8 path names, and names containing a control character (U+0000–U+001F, U+007F), are skipped, since they cannot print as one-line labels.
- **Errors.** Never print raw OS error strings. Print memex's message, the `io::ErrorKind` name and the path, so fixtures are identical across OSes.
- **git.** An enhancement, never a requirement. The toplevel is found by listing (§4.1 item 1). The log store's place (§4.1 item 3) and an entry's `branch` and `commit` (§4.3) come from subprocess `git rev-parse`; there is no git library. When git is missing or fails (exits non-zero, as for a repo it refuses under `safe.directory`), memex reads the repository layout as plain files, and only these: the nearest `.git` entry; when it is a file, its `gitdir:` line (relative to the folder holding it); that git dir's `commondir` (relative to the git dir); and its `HEAD`. It never reads refs, `packed-refs`, objects, the index or config.
  - A `.git` folder, or a `.git` file whose git dir has no `commondir` (a submodule, a `--separate-git-dir` checkout), is a main checkout: the store is the root's own.
  - A linked worktree whose common dir is named `.git` maps to the main checkout, its parent, as with git.
  - A linked worktree whose common dir has another name (a worktree of a bare repo), or whose `gitdir:` or `commondir` cannot be read, cannot be placed: `log` exits 3 with `cannot find the main checkout's log store: git did not answer and this linked worktree's repository is not a .git folder; run memex log where git works` (JSON code `io`), since the worktree's own store would be lost with it. Read verbs are unchanged.
  - `branch` comes from `HEAD` when it reads `ref: refs/heads/<name>` and no part of `<name>` starts with `.` (reftable's placeholder is `refs/heads/.invalid`); `commit` is omitted, since it needs refs.
  git is asked for the store only when the toplevel's `.git` is a file (a linked worktree, a submodule, a `--separate-git-dir` checkout); a `.git` folder means the root's own store without a git call; with no toplevel, git is asked only when a `.memex/` is in reach, so a bare repo keeps its store in its git dir (§4.1 item 3). Accepted limits: a bare repo whose folder is named `.git`, `core.bare=true` and `GIT_DIR` are not consulted there. With git present and answering, every other store and HEAD answer is git's, with one bound: the store maps to a main checkout only when the toplevel's own `.git` is a file (a linked worktree), so a nested `.git` folder that git skips never sends `log` to a store other than the root's own. Layout files are read only when they are regular files of at most 16 KiB, and a `commondir` must name a folder.
- **TTY.** `std::io::IsTerminal`. TTY stdin for `log` and INFO-CAP-02 are the behaviours with no fixture case (CI runners have no portable TTY; INFO-CAP-02's 3,000-byte boundary depends on the sandbox root's length, which differs by OS); unit tests pin them. Races and closed pipes are pinned by the opt-in stress suite (`tests/stress/`).
- **Shell examples.** Docs use bash heredocs (Git Bash on Windows runs them); stdin piping works from any shell.

## 5. Usage patterns (the way of thinking)

- **Read the compression before the source.** Go one-liner, then doc, then chain. Open code only when the intent layer can't answer.
- **Use the right tool.** `rg` finds symbols. memex finds meaning, cases and findings.
- **Write it where the next reader will look.** If it's true of the code, it goes in the header or README. If it's true of this run, it goes in a log. If it's a claim a test proves, it goes in `CASES.md` (`memex: cases`).
- **Cite instead of copying.** An id or path is enough, because memex builds the links.

**Orient.**
```sh
memex show crates/namer               # README + children one-liners + chain
memex show crates/namer/src/hash.rs   # header, cites (with claims), cited by, chain
```

**Find the tests for X.**
```sh
memex search stable class names --in cases   # the claims
memex show NAME-001                   # claim, evidence, who cites it
rg NAME-001                           # the test itself (the id is in its name or a comment)
```

**Log a finding.** Record what you learned that the code doesn't say: a verdict, a dead end, a gotcha. Do not log a diary.
```sh
memex log review --tag namer <<'EOF'
NAME-004..006 have no citing README
crates/namer/README.md describes stations but cites no ids; claims match NAME-004..006 by text only.
EOF
```

**Hand off by reference.** A sub-agent ends its run by logging the full report and returning one line, `report: impl:20261013-214107-show-chain`. The orchestrator runs `memex show <ref>` only when it needs the detail. Messages are logs with `--to`, and recipients find them with `memex search to:<id>`; `info` shows how many are waiting. Live back-and-forth stays in the harness's own messaging.

**Check intent against code.**
1. Run `memex show <dir>`, then read the code it describes.
2. For each claim, does the code do it? Is there a cited case that proves it?
3. Run `memex show <dir>` and read its `cites`: an `unresolved` entry is a broken citation, or run `memex audit <dir>`, which lists them with missing headers and READMEs.
4. Fix the README, header or collection doc in the same change, or log the mismatch to `drift` if it's out of scope.

**Change code.** In the same commit, update the header or README you invalidated, add or adjust case lines, and put the id in each new test (its name, or a comment on it). Then run `memex audit <paths>` and fix what it lists.

**Distill a log; never promote it.** Collections are authored, so never copy log files into the repo or give them `memex:` frontmatter: that turns a run's record into permanent noise. When a log holds something still true, write it once, in your own words, into the README, header or collection doc where the next reader will look, cite the ids, and let the log expire.

## 6. Agent integration

**AGENTS.md snippet (verbatim):**

```markdown
## memex: read intent before code
This repo keeps an intent layer. The graph: README.md per folder and a header comment atop each file
(first line = purpose). Dimensions: any Markdown file with `memex: <collection>` in its frontmatter
joins that collection, such as CASES.md (`memex: cases`): test ids with their claims; each id is in its test's name or in a comment on it.
- Overview: `memex info`: where everything is (graph, collections, streams, messages for you).
- Orient: `memex show <path>`: header, cites, who cites it, README chain; folders list children.
- Find: `memex search <words>` over READMEs, headers, collections and logs. Use `rg` for symbols.
- Tests for X: `memex search <words> --in cases` (or the collection `memex info` names); `memex show <ID>` gives claim and evidence; `rg <ID>` finds the test.
- Findings: `memex log <stream> [--to <agent>] <<'EOF' … EOF` prints a ref. Hand back the ref, not the report.
- Messages to you: `memex search to:<your-id>`.
- When you change code, update its header/README in the same change, then run `memex audit <paths>` and fix what it lists. Cite ids and paths; don't copy.
```

**Claude Code plugin (optional).** The memex repo is its own marketplace. The plugin contains exactly:
- **One skill (`SKILL.md`).** The snippet above, plus the §5 recipes and the conventions in §2.
- **One hook script** on SessionStart and SubagentStart. It checks that `memex` is on PATH and prints the install command if not. On SessionStart it prints `memex info`. On SubagentStart it injects (as `additionalContext`) `memex info --agent <agent_type>` and the line "your memex id is `<agent_type>`; pass `--agent <agent_type>` to `memex log` and `memex info`", where `agent_type` is the role name from the hook input. It never sets `MEMEX_AGENT`: a session-wide export would leak the parent's id into every sub-agent.

There is no MCP server, no other hooks, and no per-agent code beyond this.

### 6.1 Install

**One step**, for a human or an agent:

```sh
npx @aswemaythink/memex init
```

That installs the binary (if needed), creates `.memex/`, wires `AGENTS.md`/`CLAUDE.md`, and prints the atlas. Uninstall: delete `.memex/` and the marked block.

**Agent install prompt.** `INSTALL.md` at the repo root is a prompt a user pastes into their agent: it installs by one route of this section, runs `init`, then offers only what this document defines (the plugin, READMEs and headers, a cases collection, logs for hand-offs) and runs `audit`.

**Manual**, one command per toolchain, then `memex init`:

```sh
brew install aswemaythink/tap/memex
cargo install agent-memex
curl -LsSf https://github.com/aswemaythink/memex/releases/latest/download/agent-memex-installer.sh | sh
powershell -c "irm https://github.com/aswemaythink/memex/releases/latest/download/agent-memex-installer.ps1 | iex"
```

Or download a binary from GitHub Releases. All of these come from one cargo-dist release per tag.

**Claude Code plugin (optional):** `/plugin marketplace add aswemaythink/memex`, then `/plugin install memex@memex`.

## 7. Non-goals

- **Code.** No searching or indexing code bodies, no symbol lookup. That is `rg`'s job.
- **Infrastructure and generation.** No server, daemon, watcher, sync, auth, teams, MCP, embeddings, or LLM summarisation.
- **Mapping and linting.** No mapping files and no declared link tables, because links are derived from mentions. No linter of code: `audit` (§3.5d) reports gaps in the intent layer and file length, never parsed code metrics such as complexity or function length. No code↔test linking by reading test bodies.
- **Bespoke adapters.** No per-repo collection code: generic kinds only, and building a collection is the owning skill's job.
- **Log management.** No editing, inboxes, read state or delivery for logs. Retention is a size cap (`max_logs_mb`) plus an optional age limit (`keep_logs`), applied on write.

## 8. Overrides of the original plan

1. **`init` verb.** Creates `.memex/` with its own `.gitignore` (logs and cache ignored) and `config.json`. Logs stay in the main checkout's `.memex/logs/`, as the original plan had it, with the git common dir only as the bare-repo fallback.
2. **Built-ins renamed.** The `graph` label (opaque) becomes `readmes`, `headers` and `cases` (new), plus `logs` for all streams. reference-ui's `cases` manifest is renamed `neo-cases`. (2026-10-15: `cases` became an ordinary collection declared by frontmatter, §2.2.)
3. **Global search covers the intent layer.** This settles an open question. Grouped output solves the drowning problem that made the prototype keep the graph targeted-only.
4. **New verb `info`: the atlas.** Where things are, never contents. It replaces bare `search` as the one overview, and it's the start hook's single, size-capped call.
5. **Log body limits.** 16 KiB cap; bodies may carry their own frontmatter; logs are immutable.

## 9. Open questions (with recommendations)

1. **Spike 1: can an agent in a sandboxed worktree write to the main checkout's `.memex/logs/`?** Recommended fallback if not: write to the worktree's local `.memex/logs/`, and have reads merge across `git worktree list`.
2. **Spike 2: agent ids.** If the harness exposes no id, keep `--agent`/`$MEMEX_AGENT` and have the skill tell agents to pass their role name. Never invent ids.
3. **A `check` verb for drift?** Settled (user, 2026-10-16): `check` ships in 0.1 (§3.5d); renamed `audit` 2026-10-17. Uncited docs are not a finding; roster `<-N` covers orphans.
4. **Cache.** Settled for now (oracle, 2026-10-15): none. Opening every walked `.md` for frontmatter costs reference-ui +1-2% (info 54 → 55 ms, search 67 → 68, show 60 → 61; budgets 90/115/95) and the 100k tree +2-4% with 3% Markdown, +7-8% with 10%. Revisit when cold start exceeds 200 ms on the largest proving ground; then a stat-keyed cache, with no CLI change. Root marks (2026-10-20, §4.2 Roots) add no read: the walk already reads every README whole, so a marked root costs one frontmatter parse; an A/B against 0.1.1 on reference-ui is within noise (search 310-325 ms both, on a loaded machine).
5. **Other Markdown (`docs/`, `SPEC.md`) as intent?** Settled: a `memex:` frontmatter line puts any of it in a collection (§2.2). READMEs stay the graph.
6. **Log retention.** Settled (user, 2026-10-17): a size-capped rolling window, pruned on write; `max_logs_mb` default 10 MiB per repository, since disk space is the practical limit. `keep_logs` stays as an optional age limit, off by default.
7. **Stemming.** English only. Ids and paths are never stemmed.
8. **Case-id grammar too strict (for example lowercase ids in existing suites)?** Keep it strict. Collections can still hold non-conforming ids under `coll:id`; they just aren't citable bare.
9. **Windows.** Normalise to `/` in labels. Support it, but don't prioritise it before the first push.
