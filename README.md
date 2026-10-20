# memex

memex is a CLI that indexes a repo's authored intent layer so agents read intent before code. Markdown and a CLI only: no server, daemon, auth, MCP, LLM, vectors or database.

**Documentation:** [aswemaythink.github.io/memex](https://aswemaythink.github.io/memex/) has getting started, concepts, patterns and every verb.

## Why

Code records what a system does. The intent layer records what it is for: a README says why a folder exists, a header says what a file is for, and a case says what is claimed and how it is proven. Whoever knew the most compresses it at the moment they knew it best, and memex walks the citations between those pieces in both directions. The name is Vannevar Bush's: his 1945 essay describes a desk for linking documents into trails. This memex takes his side.

## Install

With an agent: paste [INSTALL.md](INSTALL.md) into it, in the repo you want to index. It installs memex, looks at the repo, asks you a few questions and sets up what you choose.

One step, for a human or an agent:

```sh
npx @aswemaythink/memex init
```

Manual, one command per toolchain, then `memex init` (or download a binary from GitHub Releases):

```sh
brew install aswemaythink/tap/memex
cargo install agent-memex
curl -LsSf https://github.com/aswemaythink/memex/releases/latest/download/agent-memex-installer.sh | sh
powershell -c "irm https://github.com/aswemaythink/memex/releases/latest/download/agent-memex-installer.ps1 | iex"
git clone https://github.com/aswemaythink/memex && cd memex && cargo install --path . --locked
```

The last line builds the `memex` binary from a checkout of this repo. It is one codebase for Linux, macOS and Windows.

## Quick start

In the repo you want to index, run `memex init` once:

```sh
memex init
```

It is idempotent, never overwrites existing files in `.memex/`, and reports each action. It writes:

- `.memex/`: a `.gitignore` (it ignores `logs/` and `cache/`), a `config.json` (`{"max_logs_mb": 10}`) and `collections/` for manifests.
- The memex block in AGENTS.md, between `<!-- memex:begin -->` and `<!-- memex:end -->`. It creates AGENTS.md if missing. If CLAUDE.md exists without `@AGENTS.md`, it appends that line. It never creates CLAUDE.md.
- If no `memex` is on PATH, a copy of the binary in `~/.local/bin` (`%LOCALAPPDATA%\Programs\memex` on Windows), with `add to PATH: <dir>` when needed.

Then it prints `memex info`. Reads need no setup; only `memex log` needs `.memex/`.

Now look around:

```sh
memex info                  # where everything is: graph, collections, streams, messages for you
memex show <dir>            # its README, its children's one-liners, the chain up to the root
memex show <file>           # its header, what it cites, who cites it, the chain
memex search <words>        # ranked hits, grouped by collection
memex audit                 # gaps in the intent layer: missing intent, long files, unresolved refs
```

`memex info` prints where things are, never their contents. Its shape, from design §3.5 (the counts are examples):

```
memex  /repo · log store .memex/logs/
graph        48 readmes · 912 headers
collections  cases 214 · doom 41 · neo-cases 294 · perf 212
streams      impl 12 (3 new in 24h) · review 3 (1 new) · drift 5
to cook-2    2 messages → memex search to:cook-2
here         crates/namer — Namer: turns atoms into stable class names
use          memex show <path|ref> · memex search <words> · memex log <stream> <<'EOF' … EOF
```

Search output has one group per collection, in this order: the graph (readmes, headers), the collections alphabetically, then the logs. Its shape, from design §3.2:

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

Record a finding. The body comes from stdin, and its first non-blank line is the title. Stdout is exactly the ref, on one line:

```sh
memex log review --tag namer <<'EOF'
NAME-004..006 have no citing README
crates/namer/README.md describes stations but cites no ids; claims match NAME-004..006 by text only.
EOF
```

A ref has the form `<stream>:<timestamp>-<slug>`. Pass it on, and the reader runs `memex show <ref>`.

Every printed label is a valid `show` ref. Filters are `field:value`, such as `to:cook-2` or `since:2d`. `--json` makes stdout one JSON document, errors included. Exit codes: 0 ok (zero hits included), 1 not found or ambiguous, 2 usage error, 3 environment error, such as `log` with no `.memex/`.

## The intent layer

The intent layer has two parts, tied together by citations. The **graph** sits on the tree and is found by where it is. **Dimensions** cut across the tree and are found by what they are. **Logs** are the time dimension.

### The graph

- **A README per folder.** Line 1 is `# Name`. The first paragraph says what the folder is and why it exists.
- **A header per file.** The first comment block in a non-trivial file. Its first sentence states the file's purpose.

The comment syntax comes from the file's extension: `//` for Rust, Go and TypeScript, `#` for Python, shell and YAML (a Python file with no leading `#` block uses its module docstring), `<!-- -->` for HTML and XML, and more. Files with other extensions are not harvested. A Markdown file has no header: `show` prints it in full, and a collection doc's one-liner is its first heading.

The one-liner is the first sentence of a README's first paragraph, or of a header's block. Text output cuts it at 120 characters with `…`. JSON keeps all of it.

### Dimensions

Any Markdown file joins a collection with a frontmatter line, `memex: <name>`. No manifest or config is needed. Cases are the usual first collection. A decisions doc joins the `decisions` collection:

```markdown
---
memex: decisions
status: accepted
---
# Logs live in the main checkout
Worktrees are removed when a slice lands, so their logs would go with them.
```

A `CASES.md` with `memex: cases` joins the `cases` collection. Each column-0 list item that opens with an id is one doc, called an entry:

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

- A doc's label is `<collection>:<id>`. Its id is its `id` field, or else its file stem.
- An entry's id has the form `[A-Z][A-Z0-9]*(-[A-Z0-9]+)+`, such as `NAME-001`, and is kept forever. Its claim is the rest of its first line. Backticked paths that name existing files are its evidence.
- Put each id where `rg <ID>` finds its test: in the test's name where the language allows it (`it("NAME-001 …")`), else in a comment on the test (`// NAME-001`, `# NAME-001`).
- An entry index is not itself a doc: `memex show NAME-001` prints an entry with its claim, evidence and cited by, while `memex show tests/CASES.md` prints the file.
- Collection names match `[a-z0-9][a-z0-9-]{0,39}`.
- Manifests in `.memex/collections/` cover structures that are not Markdown: a `folder` kind has one doc per folder with a marker file (such as `case.json`), and a `json` kind has one doc per entry of a JSON array.
- A manifest needs no list of folders. A README.md with `memex: <name>` in its frontmatter makes its folder a root of the manifest `<name>`, so a new folder of cases joins by its README, wherever it lives. The README stays the graph's; it is not a collection doc. An optional `roots` list in the manifest still works, and memex reads it together with the marked folders. A README whose `memex:` names no manifest gets a warning.

```json
{"name": "lib-cases", "kind": "folder", "marker": "case.json", "merge": ["README.md"], "id": "id", "title": "name"}
```

```markdown
---
memex: lib-cases
---
# Slider cases
```

### Citations and backlinks

Cite, don't copy: write ids, paths and refs, and memex derives the links both ways. There are no mapping files. These forms link:

| Write | Links to |
|---|---|
| `NAME-001` | the doc that declares that id; prose like `UTF-8` never links |
| `PREFIX-*` | every declared id with that prefix |
| `collection:id` | that doc, when `collection` is a known collection name |
| a path such as `src/hash.rs` | the file or folder it names |
| a Markdown link to a file or folder | a file or folder in the repo; URLs and `#anchors` are not refs |

`memex show` prints **cites** (each with its target's one-liner) and **cited by**. An explicit ref that resolves to nothing is listed as `unresolved`. Citations inside fenced code blocks are ignored.

### Logs

Logs are scaffolding: what a run did, useful for a couple of working days. Each entry is one file, `<YYYYMMDD-HHMMSS>-<slug>.md`, in UTC, with the frontmatter memex writes: `stream`, `time`, `agent`, `to`, `tags`, `branch`, `worktree` and `commit`. `commit` is HEAD at write time, so a reader can judge whether a finding has gone stale.

- Logs live in `.memex/logs/`, which `.memex/.gitignore` keeps out of git by default. Every worktree shares the main checkout's store.
- Entries are immutable. A correction is a new entry that cites the old ref.
- The store is a rolling window capped at `max_logs_mb` (default 10 MiB, all streams together): each `memex log` deletes the oldest entries until the store fits. An optional `keep_logs` (`Nm`, `Nh`, `Nd`, `Nw` or `forever`) also deletes entries older than that. Pruning happens only on write.
- Anything worth keeping belongs in a collection. When a stream proves durable, copy its files into the repo and add a `memex:` line to their frontmatter.

## Auditing the intent layer

`memex audit` reports where the intent layer is missing or broken, for the whole repo or for the paths and globs you name. It prints one group per kind, in this order, and a kind with no finding is left out:

- `headers`: a file with a comment syntax that is not empty and has no header.
- `readmes`: a folder with no README.md whose subtree holds a file with a comment syntax. A folder of data alone needs none.
- `length`: a file with more than 500 code lines, not counting its header and license blocks.
- `one-liners`: a header or README whose one-liner is over 100 characters.
- `thin`: a file of 80 lines or more with a header, lowest header ratio first.
- `cites`: an explicit reference that resolves to nothing. Logs are not audited.
- `ids`: a duplicate id within a collection.
- `roots`: a README whose `memex:` names no manifest, a manifest with no root, or a root nested inside another root of the same manifest (its docs would be read twice).

`thin` is a pointer, not a finding. It shows where intent is thin and has no threshold, so it is never "fixed", and it is left out of the finding count and of `nothing to fix`.

`audit` is a report, not a gate: it exits 0 whatever it finds. A path that names nothing exits 1, and a usage error exits 2. Its output looks like this (design §3.5d):

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

## Using it with agents

`memex init` writes this block into AGENTS.md (design §6):

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

The optional Claude Code plugin lives in [plugins/memex](plugins/memex/README.md): one skill that carries the snippet, the usage recipes and the conventions, and one hook script. On SessionStart the hook prints `memex info`. On SubagentStart it adds `memex info --agent <agent_type>` and gives the sub-agent its memex id. It never sets `MEMEX_AGENT`, since a session-wide value would leak the parent's id into every sub-agent.

```
/plugin marketplace add aswemaythink/memex
/plugin install memex@memex
```

## Principles

- **Markdown files and a CLI.** No server, daemon, auth, MCP, LLM, vectors or database. memex starts, reads plain files live, prints and exits.
- **Agent-agnostic.** Anything with a shell can use it. Plugins only wrap it.
- **Conventions over configuration.** A folder's README and a file's header are found by where they are. A frontmatter line puts any Markdown file in a collection.
- **memex reads intent, `rg` reads code.** It never indexes code bodies, only the comment block atop a file.
- **Bounded output.** Every list prints `showing N of M`, and `--limit` works everywhere. Nothing is capped silently.
- **Logs are scaffolding; knowledge is authored.** A log expires. What is still true gets written into a README, a header or a collection doc by someone who understands it.
- **A feature earns its place** by removing friction an agent actually hit in a playtest.

## Non-goals

- **Code.** No searching or indexing of code bodies, and no symbol lookup. That is `rg`'s job.
- **Infrastructure and generation.** No server, daemon, watcher, sync, auth, teams, MCP, embeddings or LLM summarisation.
- **Mapping and linting.** No mapping files and no declared link tables, because links are derived from mentions. No linter of code: `audit` (design §3.5d) reports gaps in the intent layer and file length, never parsed code metrics such as complexity or function length. No code-to-test linking by reading test bodies.
- **Bespoke adapters.** No per-repo collection code. Generic kinds only.
- **Log management.** No editing, inboxes, read state or delivery for logs. Retention is one setting, applied on write.

## Links

- [Documentation](https://aswemaythink.github.io/memex/): the site, with getting started, patterns and the verb reference.
- [DESIGN.md](DESIGN.md): the behaviour. It is authoritative.
- [tests/cases](tests/cases/README.md): the conformance suite that pins behaviour.

## License

MIT-licensed: see [LICENSE.md](LICENSE.md).
