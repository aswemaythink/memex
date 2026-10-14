# memex — design

> Status: spec for the Rust build. Settled decisions come from `plan.md`; overrides are listed in §8.

## 1. Thesis and principles

**memex is authored context compression.** Whoever knew the most compresses it at the moment they knew it best. A README compresses a folder, a header compresses a file, a case line compresses a test, a log compresses an agent's run, and a ref compresses a handoff. Together these form an *intent layer*: a language on top of the code that agents read to understand architecture, and check against the code to find drift. memex reads it back one level at a time: one-liner (`search`), then document (`show`), then chain (`show <path>`).

- **Markdown files and a CLI.** No server, daemon, auth, MCP, LLM, vectors or database. memex starts, reads plain files live, prints and exits.
- **Agent-agnostic.** Anything with a shell can use it. Plugins only wrap it.
- **Conventions over configuration.** The intent layer needs zero config. The only config is collection manifests.
- **memex reads intent, `rg` reads code.** memex never indexes code bodies, only the comment block at the top of a file.
- **Bounded output.** Every list prints `showing N of M`, and `--limit` works everywhere. Nothing is capped silently.
- **Logs are scaffolding; knowledge is authored.** A log is mechanical: what happened in a run, useful for a couple of working days, then junk. memex prunes it. Anything still true after that gets compressed into a README, a header, a case or a collection by someone who understands it. Automated memory tools try to keep everything forever; memex says most of it was never worth keeping.
- **A feature earns its place** by removing friction an agent actually hit in a playtest.

### Lineage

The name is Vannevar Bush's. In "As We May Think" (*The Atlantic*, July 1945) he described the memex: a desk of microfilm where a reader links documents side by side into **trails**, annotates them, and hands them to colleagues. He predicted a profession of **trail blazers** who build useful paths through the shared record. The same essay imagines a forehead camera that photographs everything. That's automatic capture, eighty years early. The memex is the other idea: associations made on purpose by someone who understands the material. That one led to Engelbart, Nelson's hypertext and the web.

This memex takes Bush's side. Citations and backlinks are the trails; developers and agents are the trail blazers; logs are the camera's film, kept a couple of days and then thrown away. Org: `aswemaythink` (pending).

## 2. The intent layer: five conventions

1. **README.md per unit folder.** Line 1 is `# Name`. The first paragraph says what the folder is and why it exists; its first sentence is the one-liner shown everywhere.
2. **Header at the top of each non-trivial file.** It is the first comment block, and its first sentence states the file's purpose. That sentence is the file's one-liner.
3. **Cases have ids and claims in `CASES.md`.** The test's own name contains the id.
4. **Cite, don't copy.** Write case ids, paths and refs into READMEs, headers and logs. memex derives links both ways from them. There are no mapping files.
5. **Intent ships with code.** A change to behaviour updates its header or README in the same commit.

**One-liners.** A README's one-liner is the first sentence of the first paragraph after the H1, or the H1 text if there is no paragraph. A header's is the first sentence of the block. A sentence ends at `.`, `!` or `?` followed by whitespace or the end of the paragraph. Text output truncates a one-liner at 120 chars with `…`; JSON keeps the full text.

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
- **Limits.** The index keeps the first 60 lines; `show` prints the whole block. A file with no header shows as `(no header)` in listings. That gap is itself a signal.
- **Files read.** Only the first 8 KiB of each file is read. Binary files are skipped. A header still open at the 8 KiB cut is shown with a final line `… (header cut at 8 KiB)`, never silently.

### 2.2 The case index: `CASES.md`

There is one standard: a Markdown file named `CASES.md`, placed anywhere (by convention next to the tests it indexes). Every list item at column 0 whose first element is a backticked case id is a case.

```markdown
# Namer cases

## Stability
- `NAME-001` Class names are stable across runs and platforms.
  Hash input is the canonical atom string, never object identity.
  Evidence: `tests/namer_stable.rs`
- `NAME-002` Two atoms with equal declarations share one class.
```

- **Case id.** `[A-Z][A-Z0-9]*(-[A-Z0-9]+)+`, unique across the repo, for example `NAME-001`, `STT-WRAP-06` or `LIB-MEASURE-001`. A case keeps its id forever; retired ids are not reused.
- **Claim.** The rest of the first line, as one sentence stating what the case proves.
- **Detail.** Indented continuation lines (optional).
- **Evidence.** Backticked paths inside the item that resolve to existing files (relative to the CASES.md folder first, then the root). This is optional, because the id in the test name already makes `rg NAME-001` exact.
- **Group.** The nearest heading above the item, recorded as the `group` field.
- **Planned cases.** A case written before its test exists is fine, and is README-driven development applied to tests. Say so in the claim, for example "(planned)".
- **Existing case structures.** Case-per-folder layouts and JSON case arrays need no conversion. A manifest (§4.2) indexes them, and their ids follow the same citation rules.

### 2.3 Citations and derived backlinks

memex scans every indexed doc (READMEs, headers, CASES.md entries, manifest docs, logs) for citations. Citations are never extracted from fenced code blocks (a fence is a line of three or more backticks or tildes indented at most three spaces; an unclosed fence runs to the end of the doc). A fence closes at a line of the same mark, at least as long, with nothing else on it; a backtick opener's info string holds no backticks. This one fence definition applies everywhere memex skips fenced code: citations, CASES.md items, markdown docs and init's markers. Self-citations are dropped.

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
memex show   <path|ref>... [--up N] [--limit N] [--json]
memex log    <stream> [--to <id>[,...]] [--tag <t>[,...]] [--agent <id>] [--json]  < body
memex info   [--agent <id>] [--json]
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
- **Other worktrees.** A path outside the current root resolves against *its own* nearest root (a `.memex/` or git toplevel above it). With neither, its root is the path itself for a folder, or its parent for a file; memex never walks the enclosing filesystem. `memex show ../wt-cook-3/src/x.rs` reads that worktree's intent layer, so no `@worktree` suffix is needed. Logs are shared anyway.
- **Built-in collections.** The names `readmes`, `headers`, `cases` and `logs` are reserved. `logs` means every stream.

### 3.2 `search`

- **No arguments.** Usage error (exit 2) pointing to `memex info`, which is the one overview.
- **Words.** Ranked BM25 with field weights, English stemming and prefix matching. Fuzzy matching allows one edit for words of 5 or more characters, and those hits are marked `[~variant]`. Ids and paths are also kept as whole tokens, so `NAME-001` matches exactly. Any word may match; more matches rank higher. Scores are only comparable within one group. Ranking compares scores quantized to `1e-9`, then by ref, so libm differences across OSes cannot reorder results.
- **Filters `field:value`.** Exact and case-insensitive, combined with AND. Reserved: `agent:`, `to:`, `tag:`, `branch:`, `worktree:` (basename or path), and `since:` (`30m`, `6h`, `2d`, `1w` or `2026-10-13`; it keeps only docs that have a `time`). Any other `x:` is a filter only if some doc in scope has field `x`; otherwise it is a literal word. That keeps `std::io` and refs searchable.
- **Grouped output, always.** Each collection with hits is one group. The fixed order is `readmes`, `headers`, `cases`, manifest collections (alphabetical), then `logs` as one group. Empty groups are omitted.
  - **Limits.** Without `--in`, `--limit` applies per group (default 5). With `--in`, the default is 20.
  - **Why groups.** Grouping is why READMEs and headers can be in global search: thousands of headers cannot drown a README, and log noise stays in its own group.
- **Filters without words.** Lists matching docs, newest first for logs.
- **`--in X` with no words.** Prints the roster: every doc's one-liner, logs newest first, everything else by id. Each roster line carries `<-N`, its cited-by count (prefix citations excluded), so `search --in cases` *is* the orphan audit (`<-0`). Default limit 50.
- **`--in` names.** Built-in collections, manifest names and stream names. An unknown name is a usage error (exit 2), never a silently empty result.

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

It takes one or more refs and prints a section for each, separated by `=== <label> ===` lines. A ref repeated in one call prints once, at its first position. A requested file that can't be read, or a requested folder that can't be listed, is that ref's error (`io`), so `show` exits 1 as for any failed ref; unreadable files met while loading the corpus stay warnings (§4.2).

- **File.** The header (full content for `.md`), then `cites`, then `cited by`, then the chain. `--up N` limits the depth; `--up 0` prints the file alone.
- **Folder.** The folder's README if there is one; otherwise `(no README)`. Then the children (the folder's own README.md excluded, since it is already printed): subfolders with their README one-liners, and files with their header one-liners or `(no header)`. Then `cites`/`cited by` for the folder path, then the chain. A folder without a README lists its children and is not an error. `show .` is the map of the root.
- **Chain.** Ancestor READMEs in full, nearest first, root last. For a README.md itself, the chain starts at the folder above its own, since the file is already printed in full.
- **Case.** Claim, detail, group, the defining `CASES.md`, evidence paths, and cited by.
- **Log or collection doc.** The full text, its fields as a `key: value` block, cites, and cited by.
- **Limits.** The children and cited-by lists use `--limit` (default 50) and always print `showing N of M`, adding `(--limit M for all)` only when the list was cut (N < M).
- **Exit code.** If any ref fails, memex still prints the others and exits 1.

### 3.4 `log`

- **Body.** Read from stdin, required and non-empty. If stdin is a TTY, memex exits 2. The maximum is 16 KiB, measured on the body as stored (after the BOM is dropped, CRLF becomes LF and the bytes are decoded), which exits 2 with the message "logs are findings: cite artifacts by path". The title is the first non-blank body line after any frontmatter, with leading `#`s and spaces stripped. If the title is empty, memex exits 2. The slug is made from the title: lowercase ASCII alphanumerics, other runs become `-`, trimmed, at most 60 chars, or `entry` if empty.
- **Store.** `log` is allowed if the root or the main checkout has `.memex/`. With neither, it exits 3 with "run `memex init`" (§4.1).
- **Extra fields.** An optional leading `---` frontmatter block in the body is merged in. Collisions with memex-owned keys exit 2. Keys compare case-insensitively, both against memex's keys and against each other, so `To:` cannot forge a `to` field. A value containing a lone carriage return exits 2.
- **Stream name.** `[a-z0-9][a-z0-9-]{0,39}`. It must not be a reserved name, a Windows device name (`con`, `prn`, `aux`, `nul`, `com0`–`com9`, `lpt0`–`lpt9`; refused on every OS so a store stays portable), or an existing manifest name (matched against the listed file names in `.memex/collections/`, exactly, never by probing). Streams need no declaration.
- **Agent id.** Taken from `--agent`, then `$MEMEX_AGENT`; otherwise it is omitted. No harness exposes a per-agent id in the environment (Spike 2), so there is no third source.
- **Output.** stdout is exactly the ref on one line, so `ref=$(memex log impl <<'EOF' … EOF)` works. With `--json`, the output is `{"ref","path"}`.
- **Immutable.** There is no edit verb. A correction is a new entry that cites the old ref, and the old entry then shows it under "cited by".

### 3.5 `info` (the atlas)

Where things are, never their contents. It prints no log bodies or titles, only names, counts and the commands to get there. It's the overview for humans and agents, and what the start hook prints. Text output is capped at **3,000 bytes**; long lists end with `+N more`.

```
memex  /repo · log store .memex/logs/
intent       48 readmes · 912 headers · 214 cases
collections  doom 41 · neo-cases 294 · perf 212
streams      impl 12 (3 new in 24h) · review 3 (1 new) · drift 5
to cook-2    2 messages → memex search to:cook-2
here         crates/namer — Namer: turns atoms into stable class names
use          memex show <path|ref> · memex search <words> · memex log <stream> < body
```

- **Header.** The root, then the log store: root-relative when the store is inside the root, else absolute.
- **Collections.** Name and doc count, alphabetical; `--json` adds kind, description and owner. Invalid manifests and duplicate case ids are listed as warnings.
- **Streams.** Total entries and how many are new in the last 24 hours, ordered by most recent activity.
- **`to` line.** A count only: every entry in the store addressed to the id. Shown when an agent id is known (`--agent`, `$MEMEX_AGENT`).
- **`here` line.** Shown only when the cwd is not the root: the nearest README's one-liner.

### 3.5b `init`

The one-step setup, idempotent. Inside git it targets the toplevel; outside git, the cwd. It reports each action.

1. Creates `.memex/` (never overwrites existing files):
   ```
   .memex/
     .gitignore          # logs/ and cache/, ignored by default
     config.json         # {"keep_logs": "3d"}
     collections/        # manifests go here (empty but for a .gitkeep, so git keeps it)
   ```
2. Writes the AGENTS.md snippet (§6) into `AGENTS.md` between `<!-- memex:begin -->` / `<!-- memex:end -->` markers, creating the file if missing and replacing the block on re-run. If `CLAUDE.md` exists without `@AGENTS.md`, it appends `@AGENTS.md` as a final line. CLAUDE.md is never created. Markers and the `@AGENTS.md` line count only outside fenced code blocks. memex never writes through a symlinked `AGENTS.md` or `CLAUDE.md`: it warns and leaves them alone.
3. If no `memex` executable is on PATH (e.g. run via `npx`; PATH entries inside a `node_modules/.bin` folder are ignored, since that is npx's temporary shim, while a global npm install counts, and on Windows `memex.exe` or `memex.cmd` count), copies its own binary to `~/.local/bin` (`%LOCALAPPDATA%\Programs\memex` on Windows) and prints the plain line `add to PATH: <dir>` if that folder isn't on PATH. The install dir comes from the `$HOME` / `%LOCALAPPDATA%` env vars, not the known-folder API, so tests can redirect it. An installed binary is replaced only if its bytes differ.
4. Prints `memex info` for the target, without an agent id (init has no `--agent`, so its output depends only on the repo).

Reads work without `init` (the intent layer needs zero config); `log` needs a `.memex/`.

### 3.6 JSON

With `--json`, stdout is always one JSON document, including errors. Without it, errors go to stderr as text.

- **Doc:** `{ref, collection, id, path, title, fields:{…}}`. `fields` holds frontmatter and manifest fields; logs add `time, agent, to[], tags[], branch, worktree, commit`.
- **`search` (with words or filters, or a roster):** `{query, filters:{…}, groups:[{collection, total, hits:[doc + {score|null, snippet, fuzzy:[]}]}]}`. Roster hits have `score: null` and `cited_by: N`, which, like `<-N`, excludes prefix citations. JSON scores are rounded to 3 decimals. The shape is the same with or without `--in`.
- **`info`:** `{root, log_store, collections:[{name, kind, builtin, count, description, owner, new_24h?, newest?, error?}], to?:{agent, count}, here?:{path, title}, warnings:[]}`, where `kind` is `log` for streams.
- **`show`:** `{docs:[doc + {text, cites:[{ref, title, resolved}], cited_by:{total, refs:[{ref, title, via?}]}, children?:{total, items:[{path, kind:"dir"|"file", title}]}, chain:[{path, text}]}], errors:[{arg, code, message, candidates?}]}`.
- **`log`:** `{ref, path}`, where `path` is absolute and `/`-separated.
- **Fatal errors:** `{error:{code, message}}`, where `code` is one of `usage | not_found | ambiguous | no_root | io`. `no_root` is `log` with no `.memex/` (§4.1).
- **Exit codes:**
  - 0: ok, including zero hits.
  - 1: not found or ambiguous.
  - 2: usage error.
  - 3: environment error: no `.memex/` for `log` (§4.1), store not writable, or an I/O failure that stops the verb.
- **Closed stdout.** If the reader closes stdout early (`memex search … | head -1`), memex stops writing, prints nothing on stderr and exits 0. A `show` ref that fails on I/O is that ref's error and exits 1 (§3.3).
- **Determinism.** Lists are sorted (ties broken by ref) unless a rule gives another order: cites and evidence keep the order they are written in. `$MEMEX_NOW` is RFC 3339 UTC with `Z`, and it replaces the clock everywhere: log time, prune, `since:`, ages and `new_24h`. An invalid value exits 3. That lets fixtures compare stdout byte for byte.
- **Ages.** Ages print as `Nm` below 60 minutes (`<60m`), `Nh` below 48 hours (`<48h`), else `Nd`.

## 4. Storage

### 4.1 Discovery

1. **Root.** memex walks up from the cwd, or from the path argument for `show`, to the nearest folder that contains `.memex/`, and the nearest one wins as with git. If there is none, the root is the git worktree toplevel, or the cwd outside git. `log` then needs `.memex/` in the root or the main checkout (item 3).
2. **Intent layer.** It is read from the root's tree, live, using the `ignore` walker. That means `.gitignore` and `.ignore` are honoured (`.ignore` is the way to exclude paths from memex and rg together). The built-in skip list is hidden entries, anything git-ignored or `.ignore`d, `node_modules` and `target`. Manifest roots may point into hidden folders. A README is matched by its listed filename, `README.md`, exactly. memex never probes with `exists()`, so behaviour is portable across case-insensitive filesystems.
3. **Log store.** `.memex/logs/`, ignored by default (`.memex/.gitignore`). Inside a git worktree it resolves to the **main checkout's** `.memex/logs/` (the same root path in the main worktree, found via `git rev-parse --git-common-dir`), so every worktree shares one store and logs survive `git worktree remove`. A bare repo has no main checkout, so its store is `<git-common-dir>/memex/logs/`; there `log` needs `.memex/` in the root only, and no `.gitignore` is written because the store is inside the git dir. `log` is allowed if the root or the main checkout has `.memex/`. With neither, it exits 3 with "run `memex init`". Creating the store also writes `.memex/.gitignore` when it's missing, so logs can never be committed by accident. memex creates the folder on the first `log`.
4. **`.memex/` contents.** Committed: `collections/<name>.json` manifests, `config.json`, `.gitignore`. Ignored by default: `logs/` and `cache/`. A team that wants to keep logs removes `logs/` from the `.gitignore`; that's their choice. `config.json` has one key, `keep_logs`. It is read from the store owner's `.memex/config.json` (the main checkout's, for a worktree), and defaults to `3d`.

### 4.2 Manifests and generic kinds

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
| `name` | all | label prefix; must equal the file stem; reserved names rejected |
| `description`, `owner` | all | shown in `info`; `owner` names the skill that builds it |
| `roots` | all | root-relative folders (required) |
| `include` | markdown, json | globs under roots (markdown default `**/*.md`) |
| `id` | all | markdown: `stem` (default) or `path` or a frontmatter field; folder: marker field or folder name (default); json: entry field (required) |
| `title` | all | field for the one-liner (default: first `#` heading / first line) |
| `weights` | all | per-field ranking boost (defaults: `id` 4, `title` 3, body 1) |
| `marker`, `merge` | folder | marker file defines a doc; `merge` globs in that folder are appended to the body |
| `at` | json | JSON Pointer to the entries array (default: document root) |

- **`markdown`.** Each matched file is one doc, with frontmatter as fields and the body as text. Matched `.json` sidecars are docs too, with their top-level scalars as fields.
- **`folder`.** Each folder containing `marker` is one doc. The marker's fields come from top-level JSON scalars, or from frontmatter for a `.md` marker. Each `merge` file adds its full text if it is Markdown, or **only its header** if it is code, so memex still never reads code bodies.
- **`json`.** Each entry of the array at `at`, in every matched file, is one doc. Scalars become fields and strings are indexed as text.
- **Frontmatter.** Flat YAML: `key: scalar`, `key: [a, b]`, or `- item` lists. Anything nested is kept as text and cannot be filtered. Frontmatter is optional, and missing fields simply don't score.
- **Unreadable files.** A README, header, CASES.md or stream folder that can't be read is a warning (`memex: warning: cannot read <path> (<ErrorKind>)`) and is skipped. It never fails a read verb.
- **Invalid manifests.** An invalid manifest or an unknown key is a warning, and that collection is skipped. It never makes the run fail. Warnings are printed once per run on stderr by every verb that loads them, in text and JSON modes. `info` also lists them, with duplicate ids, in its output.
- **Duplicate case ids.** Reported as `info` warnings, and as `ambiguous` in `show`.

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
  - **Vanished entries.** An entry or stream folder that disappears between listing and use (another process pruned it) is already gone: prune and readers skip it silently (`NotFound` never warns; every other error kind still does). A write whose stream folder is removed under it re-creates the folder and retries.
- **Cleanup.** Logs go stale, so they're pruned. Every `memex log` deletes entries older than `keep_logs` (default `3d`; `"forever"` disables pruning) across the store. `keep_logs` takes `Nm`, `Nh`, `Nd`, `Nw` or `forever`. If the value is invalid, memex prints a warning on stderr and does not prune. It never deletes on config it can't read. The age of an entry comes from its filename timestamp, and only files matching `YYYYMMDD-HHMMSS-*.md` are ever deleted. Stream folders emptied by a prune are removed. There's no background process; pruning happens only on write. Anything worth keeping beyond that belongs in a collection, not a log. `rm -rf` still works.

### 4.4 Portability

One codebase for Linux, macOS and Windows, using only cross-platform APIs. No manual Windows testing; **CI runs the fixture suite on `ubuntu`, `macos` and `windows`**.

- **File modes.** Files memex creates (log entries, init's files) get the ordinary mode for new files (0644 before umask on unix), not tempfile's private 0600; a file memex replaces keeps its mode.
- **Atomic writes.** `tempfile` + `persist_noclobber` (atomic, never overwrites) on every platform, in place of raw `link()`. If persisting fails and the target name exists in any form (Windows reports a directory there differently), the name counts as taken and the `-2`, `-3` retry runs. Deletes that the OS refuses (on Windows, a file held open by another process) are warnings, never errors.
- **Paths.** Labels and JSON always use `/`. Canonicalise with `dunce` (no `\\?\` prefixes). Compare paths case-sensitively in labels, since filesystem case behaviour varies.
- **Filenames.** Log names have no colons. Slugs are `[a-z0-9-]`, max 60 chars. Windows reserved names are moot for entry files, because the timestamp prefix comes first; stream folders refuse them (§3.4).
- **Name matching.** Special names (`README.md`, `CASES.md`, manifest files, `.memex` and everything memex reads inside it: `config.json`, `collections/`, `logs/`) are matched against directory listings, exactly. memex never decides by `exists()`, so case-insensitive filesystems behave like Linux.
- **Text.** Accept `\r\n` everywhere (headers, CASES.md, frontmatter, log bodies). A leading UTF-8 BOM is dropped on read, so frontmatter still parses from shells that add one. Write `\n`, without a BOM. The exception is a user file memex edits in place (`AGENTS.md`, `CLAUDE.md`): it keeps that file's line endings.
- **Encoding.** Non-UTF-8 file content is decoded lossily; non-UTF-8 path names are skipped.
- **Errors.** Never print raw OS error strings. Print memex's message, the `io::ErrorKind` name and the path, so fixtures are identical across OSes.
- **git.** Subprocess `git rev-parse`, no git library. If git is missing, treat the repo as non-git.
- **TTY.** `std::io::IsTerminal`. TTY stdin for `log` and INFO-CAP-02 are the behaviours with no fixture case (CI runners have no portable TTY; INFO-CAP-02's 3,000-byte boundary depends on the sandbox root's length, which differs by OS); unit tests pin them. Races and closed pipes are pinned by the opt-in stress suite (`tests/stress/`).
- **Shell examples.** Docs use bash heredocs (Git Bash on Windows runs them); stdin piping works from any shell.

## 5. Usage patterns (the way of thinking)

- **Read the compression before the source.** Go one-liner, then doc, then chain. Open code only when the intent layer can't answer.
- **Use the right tool.** `rg` finds symbols. memex finds meaning, cases and findings.
- **Write it where the next reader will look.** If it's true of the code, it goes in the header or README. If it's true of this run, it goes in a log. If it's a claim a test proves, it goes in `CASES.md`.
- **Cite instead of copying.** An id or path is enough, because memex builds the links.

**Orient.**
```sh
memex show crates/namer               # README + children one-liners + chain
memex show crates/namer/src/hash.rs   # header, cites (with claims), cited by, chain
```

**Find the tests for X.**
```sh
memex show crates/namer/src/hash.rs   # cites: NAME-001 — Class names are stable…
memex search stable class names --in cases
memex show NAME-001                   # claim, evidence, who cites it
rg NAME-001                           # the test itself (id is in its name)
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
3. Run `memex search --in cases`. Entries showing `<-0` are cases nobody cites; an `unresolved` entry in `show` is a broken citation.
4. Fix the README, header or CASES.md in the same change, or log the mismatch to `drift` if it's out of scope.

**Change code.** In the same commit, update the header or README you invalidated, add or adjust case lines, and put the id in new test names.

**Promote a stream.** When a stream has proven durable, copy its files into the repo, add a `markdown` manifest, and give the owning skill the instructions.

## 6. Agent integration

**AGENTS.md snippet (verbatim):**

```markdown
## memex: read intent before code
This repo keeps an intent layer: README.md per folder, a header comment atop each file
(first line = purpose), and CASES.md listing test ids with their claims (ids appear in test names).
- Overview: `memex info`: where everything is (collections, streams, what's new, messages for you).
- Orient: `memex show <path>`: header, cited cases, who cites it, README chain; folders list children.
- Find: `memex search <words>` over READMEs, headers, cases, collections and logs. Use `rg` for symbols.
- Tests for X: `memex show <file>` lists cited case ids; `memex show <ID>` gives claim and evidence.
- Findings: `memex log <stream> [--to <agent>] <<'EOF' … EOF` prints a ref. Hand back the ref, not the report.
- Messages to you: `memex search to:<your-id>`.
- When you change code, update its header/README in the same change and cite the case ids it relies on.
```

**Claude Code plugin (optional).** The memex repo is its own marketplace. The plugin contains exactly:
- **One skill (`SKILL.md`).** The snippet above, plus the §5 recipes and the conventions in §2.
- **One hook script** on SessionStart and SubagentStart. It checks that `memex` is on PATH and prints the install command if not. On SessionStart it prints `memex info`. On SubagentStart it injects (as `additionalContext`) `memex info --agent <agent_type>` and the line "your memex id is `<agent_type>`; pass `--agent <agent_type>` to `memex log` and `memex info`", where `agent_type` is the role name from the hook input. It never sets `MEMEX_AGENT`: a session-wide export would leak the parent's id into every sub-agent.

There is no MCP server, no other hooks, and no per-agent code beyond this.

### 6.1 Install

**One step**, for a human or an agent:

```sh
npx @rynwhite/memex init
```

That installs the binary (if needed), creates `.memex/`, wires `AGENTS.md`/`CLAUDE.md`, and prints the atlas. Uninstall: delete `.memex/` and the marked block.

**Manual**, one command per toolchain, then `memex init`:

```sh
brew install rynwhite/tap/memex
cargo install agent-memex
curl -LsSf https://github.com/rynwhite/memex/releases/latest/download/memex-installer.sh | sh
powershell -c "irm https://github.com/rynwhite/memex/releases/latest/download/memex-installer.ps1 | iex"
```

Or download a binary from GitHub Releases. All of these come from one cargo-dist release per tag.

**Claude Code plugin (optional):** `/plugin marketplace add rynwhite/memex`, then `/plugin install memex@memex`.

## 7. Non-goals

- **Code.** No searching or indexing code bodies, no symbol lookup. That is `rg`'s job.
- **Infrastructure and generation.** No server, daemon, watcher, sync, auth, teams, MCP, embeddings, or LLM summarisation.
- **Mapping and linting.** No mapping files and no declared link tables, because links are derived from mentions. No linter or `check` verb. No code↔test linking by reading test bodies.
- **Bespoke adapters.** No per-repo collection code: generic kinds only, and building a collection is the owning skill's job.
- **Log management.** No editing, inboxes, read state or delivery for logs. Retention is one setting (`keep_logs`), applied on write.

## 8. Overrides of plan.md

1. **`init` verb.** Creates `.memex/` with its own `.gitignore` (logs and cache ignored) and `config.json`. Logs stay in the main checkout's `.memex/logs/`, as plan.md had it, with the git common dir only as the bare-repo fallback.
2. **Built-ins renamed.** The `graph` label (opaque) becomes `readmes`, `headers` and `cases` (new), plus `logs` for all streams. reference-ui's `cases` manifest is renamed `neo-cases`.
3. **Global search covers the intent layer.** This settles the plan's open question. Grouped output solves the drowning problem that made the prototype keep the graph targeted-only.
4. **New verb `info`: the atlas.** Where things are, never contents. It replaces bare `search` as the one overview, and it's the start hook's single, size-capped call.
5. **Log body limits.** 16 KiB cap; bodies may carry their own frontmatter; logs are immutable.

## 9. Open questions (with recommendations)

1. **Spike 1: can an agent in a sandboxed worktree write to the main checkout's `.memex/logs/`?** Recommended fallback if not: write to the worktree's local `.memex/logs/`, and have reads merge across `git worktree list`.
2. **Spike 2: agent ids.** If the harness exposes no id, keep `--agent`/`$MEMEX_AGENT` and have the skill tell agents to pass their role name. Never invent ids.
3. **A `check` verb for drift?** Not yet. Roster `<-N` counts plus `unresolved` cover it. Add it only if playtest 2 shows agents computing orphans by hand.
4. **Cache.** None until cold start exceeds 200 ms on the largest proving ground. Then add an mtime-keyed cache in `.memex/cache/` (ignored), with no CLI change.
5. **Other Markdown (`docs/`, `SPEC.md`) as intent?** No built-in. Add a `markdown` manifest per repo; READMEs stay the convention.
6. **Log retention.** Settled: prune on write, `keep_logs` default 3 days (a couple of working days). Revisit the default after playtests.
7. **Stemming.** English only. Ids and paths are never stemmed.
8. **Case-id grammar too strict (for example lowercase ids in existing suites)?** Keep it strict. Manifests can still index non-conforming ids under `coll:id`; they just aren't citable bare.
9. **Windows.** Normalise to `/` in labels. Support it, but don't prioritise it before the first push.
