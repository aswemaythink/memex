---
name: memex
description: Read a repo's intent layer with memex before its code. It shows where things are, what a folder or file is for, which tests pin it, and how to log a finding. Use it in any repo with a .memex folder, and when asked about a repo's structure, tests or past findings.
---

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

## Your memex id

When the memex plugin starts you as a sub-agent, your context says: "your memex id is <id>". Pass that id as `--agent <id>` to `memex log` and `memex info`, so your logs and the messages to you carry it. If your context has no such line, pass your role name as the id, for example `--agent cook`.

## Usage patterns

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

## Conventions

1. **README.md per unit folder.** Line 1 is `# Name`. The first paragraph says what the folder is and why it exists; its first sentence is the one-liner shown everywhere.
2. **Header at the top of each non-trivial file.** It is the first comment block, and its first sentence states the file's purpose. That sentence is the file's one-liner.
3. **Cases have ids and claims in `CASES.md`.** The test's own name contains the id.
4. **Cite, don't copy.** Write case ids, paths and refs into READMEs, headers and logs. memex derives links both ways from them. There are no mapping files.
5. **Intent ships with code.** A change to behaviour updates its header or README in the same commit.

**One-liners.** A README's one-liner is the first sentence of the first paragraph after the H1, or the H1 text if there is no paragraph. A header's is the first sentence of the block. Text output truncates a one-liner at 120 chars with `…`.

**Headers.** The comment syntax follows the file's extension (`//` for Rust, Go and JS, `#` for Python, shell and YAML, `<!-- -->` for HTML and XML, and others). After blank lines, a shebang, pragma lines and one leading license block, the first comment block is the header; it ends at the first line that is neither comment nor blank. memex reads the first 8 KiB of each file. A file with no header shows as `(no header)`.

**CASES.md.** Each list item at column 0 whose first element is a backticked case id is a case. Its claim is the rest of the first line. Indented lines after it are detail. A backticked path inside the item that exists is its evidence.

```markdown
# Namer cases

## Stability
- `NAME-001` Class names are stable across runs and platforms.
  Hash input is the canonical atom string, never object identity.
  Evidence: `tests/namer_stable.rs`
- `NAME-002` Two atoms with equal declarations share one class.
```

A case id is unique across the repo, and a retired id is never reused. A case written before its test exists says "(planned)" in its claim.

**Citations.** memex derives links from what the docs write:

| Form | Matches |
|---|---|
| bare id `NAME-001` | a declared case id; prose like `UTF-8` never links |
| `NAME-*` | every declared id with that prefix, so a hub README can cite a family |
| `collection:id` | that doc, when the collection is known |
| path token | the file or folder it names |
| `[text](path)` | a file or folder in the repo |

Citations are never read from fenced code blocks. An explicit reference that resolves to nothing shows as `unresolved` in `memex show`.

**Logs are scaffolding.** A log is what happened in one run. memex prunes logs older than `keep_logs` (3d by default). Anything still true after that belongs in a README, a header or CASES.md.
