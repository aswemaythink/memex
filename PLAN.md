# memex: build plan

Why: [STORY.md](STORY.md). Bush's trail against his camera: authored context compression.
What: [design.md](design.md) is authoritative on behaviour. This file is how memex gets built, and it replaces plan.md (deleted; the old one is in git history once committed).
Status: blessed by the oracle 2026-10-13. In progress. Track progress in §11.

## 0. Ground rules
- **Principles (design §1).** Markdown and a CLI only. No server, daemon, MCP, LLM, vectors or database. Agent-agnostic, minimal surface. A feature earns its place through playtest friction plus a user-approved design.md edit. Logs are scaffolding; knowledge is authored.
- **Order of change.** design.md first, then CASES.md, then code. If a cook finds design.md silent or contradictory, it stops and reports. It never invents behaviour.
- **Pinning.** Every behaviour is pinned by a conformance case (§2). Unit tests are welcome for internals but don't count as pinning.
- **Intent ships with code.** Every file has a header whose first sentence is its purpose and which cites the case ids it implements (`Cases: SRCH-FUZZY-*`). Every folder has a README.md whose first paragraph says what the folder is.
- **Prior art.** akitaonrails/ai-memory proves demand (server, auto-capture, auth). memex is the opposite end. Don't drift toward it.
- **Playtest 1 frictions** (prototype, 2026-10-13) are already designed out:
  - `show <dir>` without a README → design §3.3
  - hub README backlinks → §2.3
  - silent caps → §1 bounded output
  - opaque `graph`/`cases:` labels → §3.1
  - noisy global search → grouped output, §3.2

  Playtest 2 checks that each is gone.

**Names** (one table; S8.1 updates design §6.1 and README to match):

| thing | name |
|---|---|
| GitHub repo | `aswemaythink/memex`; fallback `rynwhite/memex` until the org exists |
| crate / binary / lib | `agent-memex` / `memex` / `memex` |
| npm | `@aswemaythink/memex` if the scope is obtainable, else `@rynwhite/memex` |
| brew | `aswemaythink/tap/memex` (repo `aswemaythink/homebrew-tap`) |
| site | `aswemaythink.github.io/memex` |
| license | MIT OR Apache-2.0 |

## 1. Design fixes before build
Approved by the user and folded into design.md in S0.1 (2026-10-13). design.md is now authoritative; this list stays as the record.
1. **§3.5 example store path.** `.git/memex/logs/` → `.memex/logs/`. The `info` header prints the store root-relative when it is inside the root, else absolute.
2. **§4.2 duplicate ids.** "Warnings in bare `search`" contradicts §3.2, where bare search is a usage error. Duplicates are `info` warnings, and `ambiguous` in `show`.
3. **§3.2 stray note.** Delete the "`--around`: cut." bullet.
4. **§3.5 `to` line.** Counts every entry in the store addressed to the id ("N messages"). Drop "in 7d": retention is 3d.
5. **Log store without `.memex/` in the main checkout.** `log` is allowed if the root or the main checkout has `.memex/`. Creating the store also writes `.memex/.gitignore` when it's missing, so logs can never be committed by accident. With neither, exit 3 "run `memex init`".
6. **`keep_logs`.**
   - Read from the store owner's `.memex/config.json`; default `3d`.
   - Grammar: `Nm|Nh|Nd|Nw|forever`. If invalid: stderr warning and **no pruning** (never delete on config you can't read).
   - Age comes from the filename timestamp. Only files matching `YYYYMMDD-HHMMSS-*.md` are ever deleted. Stream folders emptied by a prune are removed.
7. **Log title and slug.**
   - Title: the first non-blank body line after any frontmatter, leading `#`s and spaces stripped. Empty → exit 2.
   - Slug: lowercase ASCII alphanumerics, other runs become `-`, trimmed, at most 60 chars, `entry` if empty. Windows reserved names are moot because the timestamp prefix comes first.
8. **`$MEMEX_NOW`.** RFC 3339 UTC with `Z`. It replaces the clock everywhere: log time, prune, `since:`, ages, `new_24h`. An invalid value exits 3.
9. **One-liners.**
   - README: the first sentence of the first paragraph after the H1, or the H1 text if there is no paragraph.
   - Header: the first sentence of the block.
   - A sentence ends at `.`, `!` or `?` followed by whitespace or the end of the paragraph.
   - Text output truncates at 120 chars with `…`; JSON keeps the full text.
10. **Chain order.** Nearest README first, root last. For a README.md itself, the chain starts at its parent folder.
11. **Citations.**
    - Never extracted from fenced code blocks.
    - Path token: a maximal run of `[A-Za-z0-9._/@+-]` containing `/` or a `.ext`, with trailing `.,:;)` stripped. It is resolved against the walked file set (no filesystem probes), must stay inside the root, and a leading `/` means root-relative.
    - Explicit refs: markdown links (not URLs or `#anchors`) and `coll:id` with a known collection. Only explicit refs can be `unresolved`.
    - Self-citations are dropped.
12. **Prefix citations (`X-*`).** Listed in `cited by` marked `via X-*`, but excluded from roster `<-N`. That keeps `<-0` working as the orphan audit even when hub READMEs cite whole families.
13. **Scores.** JSON scores are rounded to 3 decimals. Ranking compares scores quantized to 1e-9, then ref, so libm ulp differences across OSes can't reorder goldens.
14. **CLI edge cases.**
    - "A flag with no effect" is judged on the verb's grammar, not on what the refs resolve to.
    - `--json` anywhere before `--` makes usage errors JSON.
    - Bare `memex` → usage on stderr, exit 2.
15. **README detection.** Matches the listed filename `README.md` exactly, never an `exists()` probe, so behaviour is portable across case-insensitive filesystems. The built-in skip list is hidden entries, anything git-ignored or `.ignore`d, `node_modules` and `target`.
16. **`init` rulings.**
    - Inside git, it targets the toplevel; outside git, the cwd.
    - The CLAUDE.md import is appended as a final `@AGENTS.md` line. CLAUDE.md is never created.
    - "On PATH" means any `memex` executable on PATH.
    - The install dir comes from the `$HOME` / `%LOCALAPPDATA%` env vars (not the known-folder API, so tests can redirect it).
    - An installed binary is replaced only if its bytes differ.
    - The PATH hint is the plain line `add to PATH: <dir>`.
17. **Errors.** Never print raw OS error strings. Print memex's message, the `io::ErrorKind` name and the path, so goldens are identical across OSes.
18. **Warnings** (invalid manifest or config). Printed once per run on stderr by every verb that loads them, in text and JSON modes. `info` also lists them (and duplicate ids) in its output.
19. **Untested behaviour.** TTY stdin for `log` is the one behaviour with no case (no portable TTY in the runner); a unit test pins it. Non-UTF-8 file content is decoded lossily; non-UTF-8 path names are skipped.
20. **JSON and ages.** `log --json` `path` is absolute and `/`-separated. Ages print as `<60m` → `Nm`, `<48h` → `Nh`, else `Nd`.

## 2. Conformance suite (the reference-compiler bar)
**Layout.** Family folders keep parallel cooks on disjoint files and keep the claim index in memex's own convention:
```
tests/cases/README.md                    the convention below (hub; cites every family X-*)
tests/cases/<family>/README.md           what the family covers, design §, prefix
tests/cases/<family>/CASES.md            the claim index: one entry per case
tests/cases/<family>/<ID>/case.json      run spec (required)
tests/cases/<family>/<ID>/expect.txt     golden transcript (required; written by bless)
tests/cases/<family>/<ID>/fixture/       sandbox tree (optional); path parts `dot_x` → `.x`
tests/cases/<family>/<ID>/<stdin file>   only if case.json names it in `stdin_file`
```
- **Id.** `^[A-Z]+-[A-Z]+-\d{2}$`, as in reference-ui, for example `SRCH-FUZZY-01`. The first segment is the family prefix; the topic matches a `## TOPIC` heading in that CASES.md.
- **Entry format.**
  ```
  - `SRCH-FUZZY-01` <one-sentence claim>.
    <2-space indented detail: scenario and design §>
  ```
  A claim containing `(planned)` has no folder yet: write planned cases first.
- **Why no per-case README.** It would cite its own id and make every case look cited. CASES.md is the single declaration, and memex indexes it as the built-in `cases` collection (`cases:SRCH-FUZZY-01`, bare `SRCH-FUZZY-01`).

**case.json** (unknown keys are an error):
```json
{ "git": "commit",
  "setup": [{"run": ["git","worktree","add","../wt","-b","feat"]}],
  "env": {"MEMEX_AGENT": "cook-2"},
  "steps": [{"args": ["log","impl","--tag","show"], "stdin": "Title\nbody\n", "cwd": "wt"},
            {"args": ["show","impl:20261013-120000-title"]}],
  "snapshot": ["repo/.memex/logs"], "memex_on_path": false, "platforms": ["unix"] }
```
- **`git`.** `init` or `commit`. It runs in `repo/` with branch `main`, fixed identity and dates (`2026-10-13T11:00:00Z`), so commit hashes are stable.
- **`setup`.** argv arrays whose first element is `git` or `memex` only. Output is not golden'ed; a nonzero exit fails the case.
- **`steps`.** At least one. Each takes `args`, `stdin` or `stdin_file` (default: empty stdin), `cwd` (relative to the sandbox, default `repo`) and `env`.
- **`snapshot`.** Sandbox paths whose files are dumped after the last step.
- **`platforms`.** Use only where a behaviour is truly OS-specific (permissions, install dirs).

**Sandbox.**
- `fixture/` is copied to `$SANDBOX/`, created under `std::env::temp_dir()`. The runner fails fast if any ancestor holds `.memex/`.
- Environment: the inherited env minus `MEMEX_*`, `GIT_*`, `CLAUDE*` and the Spike-2 harness vars. Then it sets:
  - `HOME`, `USERPROFILE` and `XDG_CONFIG_HOME` under `$SANDBOX/home`, and `LOCALAPPDATA=$SANDBOX/home/AppData/Local`
  - `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=<empty file>`, `GIT_CEILING_DIRECTORIES=$SANDBOX`
  - `NO_COLOR=1`, `MEMEX_NOW=2026-10-13T12:00:00Z` (a step's `env` may override it)
- PATH entries that contain a `memex` executable are removed (the dogfood install must not leak in). `memex_on_path` prepends the built binary's directory instead.
- The binary is `env!("CARGO_BIN_EXE_memex")`.

**Golden `expect.txt`.** The runner renders the actual result into this text and byte-compares the whole file. Nothing is parsed, so the format only needs to be injective:
```
$ memex log impl --tag show            [cwd wt]
exit 0
stdout
| impl:20261013-120000-title
stderr (empty)
== snapshot repo/.memex/logs
-- impl/20261013-120000-title.md
| ---
| stream: impl
\ no newline at end
```
- Every output line is prefixed `| `. A missing final newline is marked `\ no newline at end`. Binary files render as `(binary)`.
- **Redactions** (the only ones allowed): the sandbox path → `$SANDBOX`, and the crate version → `$VERSION`.
  - Redact every spelling of the sandbox path: raw, canonical (macOS `/private/var`, Windows 8.3 `RUNNER~1` expanded), `\` and `/` forms.
  - Goldens may contain no other placeholder and no wildcard.

**Conformity rules.** These are separate trials, run before any case, and are never blessable. Each message states the fix.
1. Every entry under `tests/cases/` is `README.md` or a family dir. A family dir holds `README.md`, `CASES.md` and case dirs only.
2. Every CASES.md parses. Ids match the pattern, share one prefix per family, the prefix is unique across families, and ids are unique repo-wide.
3. **Bijection.** Every case dir has exactly one entry, and the folder name equals the id. Every entry without `(planned)` has a dir; every `(planned)` entry has none.
4. A case dir holds only `case.json` (valid against the schema), `expect.txt`, `fixture/` and named stdin files.
5. Goldens use only the allowed placeholders.
6. At least one case exists.

**Runner.**
- Files: `tests/conformance.rs` (`harness = false`, libtest-mimic, one trial per case id, parallel) and `tests/harness/{discover,conform,sandbox,transcript,bless}.rs`. Diffs use `similar`.
- `tests/runner_selftest.rs` runs the conformity rules over deliberately broken trees in `tests/runner-selftest/`, one per rule, and asserts each exact message (as reference-ui's `caseIndex.test.ts` does).

**Bless.**
- `MEMEX_BLESS=1 cargo test --test conformance -- <filter>` rewrites `expect.txt` for the selected failing cases only and prints the files written.
- A missing `expect.txt` fails with "run MEMEX_BLESS=1 …".
- Bless is refused when `CI` is set.
- Review is `git diff tests/cases`. A cook may only bless the ids its brief names; the orchestrator checks `git show --stat`.

**Families** (dir, prefix, design §, must cover; the coverage audit in S5.4 checks every sentence of design.md against these):

| dir | prefix | § | must cover |
|---|---|---|---|
| cli | CLI | 3, 3.6 | help after every verb; `--`; unknown/repeated/misplaced/no-effect flags; bare `memex`; unknown verb; `--version`; JSON error envelope per code; exit codes 0/1/2/3 |
| discovery | DISC | 4.1 | nearest `.memex/` wins; git-toplevel and cwd fallbacks; read-only `log` exit 3; worktree→main-checkout store; nested root mapping; bare repo store; store creation writes `.gitignore` (fix 5); git missing |
| log | LOG | 3.4, 4.3 | body required, 16 KiB message, title/slug (fix 7); frontmatter merge and collision exit 2; stream grammar, reserved and manifest names; agent precedence; `--to`/`--tag` lists; branch/worktree/commit fields; stdout ref and `--json`; `-2` collision; prune (fix 6), `forever`, invalid config; dotfiles invisible |
| header | HDR | 2.1 | one case per comment family incl. docstring and Makefile/Dockerfile; shebang, pragma and license skip; block end; 60-line index vs full `show`; 8 KiB; binary; unknown ext; `(no header)`; CRLF; first sentence |
| intent | INT | 2, 4.1.2 | README one-liner (fix 9); no-H1 README; `.gitignore`/`.ignore`/hidden/`node_modules`/`target` skipped; exact `README.md` (fix 15) |
| caseindex | CASE | 2.2 | column-0 `-`/`*` items; claim; detail; evidence (CASES dir, then root); group; planned; lowercase rejected; duplicates |
| manifest | MAN | 4.2 | markdown/folder/json kinds; frontmatter forms, nested as text; sidecars; `id`/`title`/`at`/`merge` (code merges header only); weights; name≠stem, reserved, unknown key, bad JSON → warning and skip; roots in hidden dirs |
| cite | CITE | 2.3 | declared bare ids; `UTF-8` not linked; `X-*`; `coll:id`; path tokens and links (fix 11); unresolved; folder targets; fenced code ignored; prefix citers in `cited by`, excluded from `<-N` (fix 12) |
| ref | REF | 3.1 | label round-trip per doc kind; absolute; cwd-vs-root ambiguous; `coll:id`; bare id exact then case-insensitive; basename; candidates exit 1; other-worktree path; reserved names |
| show | SHOW | 3.3 | file/md/folder/`.`/case/log sections; children and `(no README)`; `--up N`/`0`; limits line; `===` separators; partial failure exit 1; JSON shape |
| search | SRCH | 3.2 | no-args exit 2; BM25 and weights; stem; prefix; fuzzy marker; ids/paths as tokens; group order and omission; limits 5/20/50; `--in` lists and `logs`; reserved filters; `since:` forms; literal `x:`; filter-only newest first; roster `<-N`; JSON; ties by ref |
| info | INFO | 3.5 | all lines; 3,000-byte cap `+N more`; collection counts and warnings; stream order, `new_24h`; `to` line (fix 4); `here` line; no-root case; JSON |
| init | INIT | 3.5b | fresh tree; never overwrites; AGENTS.md create/insert/replace; CLAUDE.md import once, never created; subfolder → toplevel; self-install unix/windows, PATH hint, skip when on PATH; prints info; re-run unchanged |
| port | PORT | 4.4 | CRLF everywhere; `/` labels on Windows; noclobber atomic write; lossy decode; no OS error strings (fix 17) |

## 3. Dogfood: memex builds memex
- **Commit one carries the intent layer:**
  - READMEs for root, `src/`, `tests/` and `tests/cases/`, and headers on every file.
  - A hand-made `.memex/` identical to what `init` will write (`.gitignore`, `config.json`, `collections/.gitkeep`).
  - AGENTS.md: the design §6 snippet between memex markers, plus a "Working on memex" section (these rules, the brief template, golden policy).
  - CLAUDE.md containing `@AGENTS.md`.
  - `.ignore` hides `tests/cases/*/*/fixture/` and `tests/runner-selftest/` from memex and rg (use `rg -u` to search them).
- **The Node prototype is not used.** Its ROOT is hardwired to reference-ui, it has no `log`, and it teaches a stale grammar. Before G1, the repo is small enough for `rg` and the READMEs.
- **Before G1:** cooks report through the harness as usual. Spike results are written into §5 of this file.
- **G1 (end of S1.1):**
  - After every landed slice that changes behaviour, the orchestrator runs `cargo install --path . --locked`. The dogfood `memex` is always the last landed main, never a cook's work in progress.
  - Every cook ends with `memex log impl --agent cook-<slice> --tag <family> <<'EOF' … EOF` and returns `report: <ref>` plus the tail of its DONE output.
  - Streams: refiner → `review`, oracle → `oracle`, break-it → `drift`, spikes → `spike`, playtest → `playtest`.
  - The orchestrator reads the log file directly until S2.0, then uses `memex show <ref>`.
  - The first worktree cook's log must appear in the main checkout's `.memex/logs/`; verify it. That is live evidence for Spike 1.
- **From S2.0:** before each brief, the orchestrator runs `memex show <paths in scope>` and pastes the one-liners instead of re-explaining.
- **From S3.4:**
  - `memex search` replaces scouting for intent questions.
  - `memex search --in cases` is the orphan check.
  - `tests/selfhost.rs` gates CI by running the built binary on this repo:
    - every case is directly cited at least once;
    - every harvestable file under `src/` and `tests/` (minus fixtures) has a header;
    - every folder has a README;
    - no `unresolved` appears in `show --json` of `src/`.
- **From S4.2:** `memex init` runs on itself and must report everything unchanged; commit any block diff. `memex info` opens every session.
- **From S6.1:** the plugin is enabled for this repo from its local path, and the hook prints `info`.
- **Friction.** Any memex fumble while building memex is logged to `playtest` and triaged like playtest friction.
- **End of each phase:** a promote-or-drop pass over the logs. What's still true goes into READMEs, headers or CASES.md; the rest expires (3d).

## 4. Crate layout
One package, `agent-memex`: `[[bin]] memex`, lib `memex`, `[[test]] conformance` with `harness = false`, `rust-version = "1.85"` (the dev-deps need 1.85), `exclude = ["tests/","site/","plugins/"]`.
- **Deps:** `ignore` (and its `globset`), `serde`, `serde_json`, `rust-stemmers`, `tempfile`, `dunce`.
- **Dev-deps:** `libtest-mimic`, `similar`.
- **Hand-rolled on purpose:** argv grammar (exact help, error text and JSON envelopes; no clap churn in goldens), flat-YAML frontmatter, UTC civil time, BM25F (no tantivy, no index files).

```
src/main.rs            exit code from memex::run
src/lib.rs             module map
src/cli.rs             grammar, help texts                CLI
src/error.rs           codes, exits, JSON envelope        CLI
src/clock.rs           MEMEX_NOW, durations, ages
src/config.rs          .memex/config.json
src/root.rs            root, git subprocess, store path   DISC
src/walk.rs            ignore walker, skip list           INT
src/intent/{header,readme,cases}.rs                       HDR INT CASE
src/collections/{manifest,frontmatter,markdown,folder,json}.rs   MAN
src/logs/{write,read,prune}.rs                            LOG
src/corpus.rs          Doc model, loads all sources, label map
src/cite.rs            citations, backlinks               CITE
src/resolve.rs         ref resolution                     REF
src/index.rs           tokenize, stem, BM25F, prefix, fuzzy   SRCH
src/verbs/{search,show,log,info,init}.rs
src/render.rs          text and JSON output, limit lines
assets/agents-snippet.md   included with include_str!; the single source for init, README and SKILL.md
```

**Search constants** (pinned by cases; the refiner may tune them with evidence):
- BM25 k1 = 1.2, b = 0.75.
- Field weights: id 4, title 3, body 1.
- Match multipliers: exact or stem 1.0, prefix 0.5 (query words of 3 or more chars), fuzzy 0.3 (one edit, 5 or more chars).

## 5. Spikes (run in parallel with Phase 0, after commit one)
- **Spike 1: worktree to main-checkout writes.**
  - Brief: a Haiku cook with `isolation: "worktree"` and the default sandbox does not commit. It runs `git rev-parse --git-common-dir --show-toplevel`, writes a dot-temp file into `<main>/.memex/logs/spike/`, renames it no-clobber, and reports the exact errors.
  - Rulings:
    - (a) It works → design stands.
    - (b) Blocked only by the harness sandbox, and an allow-rule fixes it → design stands; document the rule in the README and SKILL.
    - (c) Nothing fixes it → design §9.1 fallback (worktree-local store, reads merged across `git worktree list`). **Consult the oracle before adopting (c)**, since it breaks "logs survive worktree removal".
  - Result (2026-10-13): **(a), provisional; design stands.** From a worktree agent, `mkdir -p <main>/.memex/logs/spike` succeeded (no sandbox error). The file writes were refused by the harness's *command checker* (it could not parse a `printf` with `---` and a `$(git …)` form), not by the sandbox, and that doesn't affect memex, whose writes happen inside its own process. A rerun with plain `touch`/`cp`/`ln` (from the main checkout, since the worktree had already been removed) showed create, write, hard-link and no-clobber `EEXIST`. Final confirmation is the G1 check: the first worktree cook's `memex log` must land in the main checkout's `.memex/logs/`.
- **Spike 2: agent ids.**
  - A `claude-code-guide` agent answers: which env vars a sub-agent's Bash sees; the SubagentStart and SessionStart hook input fields; and whether a hook can export env to the agent's shell (`CLAUDE_ENV_FILE`).
  - A scout sub-agent prints the *names* of `CLAUDE*`/`AGENT*` vars, then the values of non-secret candidates only.
  - Optionally, a temporary SubagentStart hook in `.claude/settings.local.json` dumps its stdin to the scratchpad. The user must approve it first; remove it afterwards.
  - Output: the harness-variable list for the `--agent` chain (design §3.4), and whether the hook sets `MEMEX_AGENT` (§6). If no id is exposed, the skill tells agents to pass their role name. Never invent ids.
  - Result (2026-10-13, Claude Code desktop 2.1.295; live hook dump + docs + sub-agent env dump):
    - **Harness-variable list: empty.** No env var identifies an agent. A sub-agent's shell sees `CLAUDECODE=1`, `CLAUDE_CODE_CHILD_SESSION=1`, `CLAUDE_CODE_ENTRYPOINT`, `AI_AGENT=claude-code_<ver>_agent` (a product tag) and `CLAUDE_CODE_SESSION_ID`, which equals the *parent* session id, so it is shared by every agent and unusable. The `--agent` chain is `--agent`, then `$MEMEX_AGENT`, then nothing.
    - **SubagentStart hook input** (observed): `session_id, transcript_path, cwd, scratchpad_dir, prompt_id, agent_id, agent_type, hook_event_name`. `agent_type` is the role name (`cook`, `scout`); `agent_id` is an opaque per-run id.
    - **The hook does not set `MEMEX_AGENT`.** `CLAUDE_ENV_FILE` exists for SessionStart (not SubagentStart, and unset in a sub-agent's shell); a session-wide export would leak the parent's id into every sub-agent.
    - **What the hook does instead:** on SubagentStart it injects, via `hookSpecificOutput.additionalContext` (capped at 10,000 chars), the output of `memex info --agent <agent_type>` plus one line: "your memex id is `<agent_type>`; pass `--agent <agent_type>` to `memex log` and `memex info`". On SessionStart it prints `memex info` (stdout becomes context). The skill tells agents with no hook to pass their role name. design §6 is updated to match in S6.1.

## 6. Phases and slices
**DONE(F)**, run by every slice:
```
cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings   # Windows arms compile clean
cargo test --locked                                  # whole suite stays green
cargo test --locked --test conformance -- F          # the slice's cases, each listed
git show --stat HEAD                                 # only the brief's files + its tests/cases/<family>/
```
Every slice writes its CASES.md entries (from the design § and the §2 must-cover list) **before** fixtures and code. Wire-up stubs land in the earlier slice, so parallel slices touch disjoint files. ∥ means parallel, in worktrees.

**Phase 0: foundation**
- **S0.1 Skeleton and intent layer.** Orchestrator or one cook. Covers everything in §3 "commit one", plus:
  - `git init` is done already; first commit includes STORY.md, design.md (with the §1 fixes folded in), PLAN.md
  - Cargo.toml, stub `main.rs`/`lib.rs`
  - LICENSE-MIT, LICENSE-APACHE
  - `.gitattributes`: `* text=auto eol=lf`; `tests/cases/** -text`; `tests/runner-selftest/** -text`

  Done: `cargo build`; `git show --stat HEAD`.
- **S0.2 Conformance runner.** All of §2's runner, self-tests and `tests/cases/README.md`, plus the seed family `cli/` with `CLI-VERSION-01` (`--version` → `memex $VERSION`; `main.rs` prints it).
  - Done: DONE(CLI-VERSION).
  - `cargo test --test runner_selftest` is green.
  - Bless, re-run: byte-identical.
  - Hand-corrupt a fixture: it fails with the fix message.
  - **Refiner pass required.**
- **S0.3 CLI grammar and errors** ∥ S0.4. Files: `cli.rs`, `error.rs`, `main.rs`. Help texts for all five verbs. Unimplemented verbs exit 2 "not implemented" (no cases). Done: DONE(CLI-).
- **S0.4 CI.** `.github/workflows/ci.yml`:
  - matrix ubuntu, macos, windows; fmt (ubuntu only), clippy, test;
  - `git diff --exit-code` after the tests;
  - fail if `MEMEX_BLESS` is set.

  It runs once a GitHub remote exists. **User checkpoint:** create a private repo at the end of Phase 1, so Windows bugs surface early.

**Phase 1: the log loop (dogfood starts)**
- **S1.1 Clock, root, store and minimal `log`.** Files: `clock.rs`, `root.rs`, `config.rs` (read only), `logs/write.rs`, `verbs/log.rs`. This slice:
  - writes the body to the store with memex frontmatter (stream, time, branch, worktree, commit);
  - prints the ref, and `{ref,path}` with `--json`;
  - writes the noclobber temp file;
  - adds a no-op `prune::run` hook.

  Cases: all of DISC (observed through `log --json`) plus LOG-WRITE. Done: DONE(DISC-), DONE(LOG-WRITE). **Gate G1** (§3).
- **S1.2 Log rules** ∥ S1.3. File: `logs/write.rs`. Covers validation, frontmatter merge, stream names, agent chain (Spike 2), `--to`/`--tag`, slug, collisions. Done: DONE(LOG-) minus PRUNE.
- **S1.3 Prune** ∥ S1.2. Files: `logs/prune.rs`, `config.rs`. Implements fix 6. Done: DONE(LOG-PRUNE). **Refiner pass required** (it deletes files).

**Phase 2: read model and `show`**
- **S2.0 Doc model and `show` skeleton.** Files: `corpus.rs`, `walk.rs` (basic), `resolve.rs` (path + exact `coll:id`), `logs/read.rs`, `collections/frontmatter.rs`, `render.rs`, `verbs/show.rs`.
  - Stubs with final signatures: `intent/*`, `collections/{manifest,markdown,folder,json}.rs` (including the Manifest struct and kind dispatch), `cite.rs`.
  - Cases: SHOW (using `//` headers and log docs).
  - Done: DONE(SHOW-). Light refiner pass.
- **S2.1 Headers** (`intent/header.rs`) ∥ **S2.2 READMEs and walk** (`intent/readme.rs`, `walk.rs`) ∥ **S2.3 CASES.md** (`intent/cases.rs`; observed via `show cases:ID`) ∥ **S2.4 Manifests, markdown kind** (`manifest.rs`, `markdown.rs`) ∥ **S2.5 Folder and json kinds** (`folder.rs`, `json.rs`).
  - Done: DONE(HDR-), DONE(INT-), DONE(CASE-), DONE(MAN-) respectively. S2.4 and S2.5 split MAN by topic: LOAD/MD vs FOLDER/JSON.

**Phase 3: links and search**
- **S3.1 Citations** (`cite.rs`, cites/cited-by in `show.rs`) ∥ **S3.2 Full resolution** (`resolve.rs`). Done: DONE(CITE-), DONE(REF-).
- **S3.3 Search core.** Files: `index.rs`, `verbs/search.rs`. Cases: SRCH-RANK, STEM, PREFIX, FUZZY, GROUP, LIMIT, JSON. **Refiner pass required.**
- **S3.4 Filters, roster, `since:`, ages, `<-N`,** plus `tests/selfhost.rs` (§3). Comes after S3.1 and S3.3. Done: DONE(SRCH-), and `cargo test --test selfhost`.

**Phase 4: atlas and setup**
- **S4.1 `info`** (`verbs/info.rs`). Done: DONE(INFO-).
- **S4.2 `init`** (`verbs/init.rs`, `assets/agents-snippet.md`), after S4.1. Done: DONE(INIT-); then `memex init` in this repo, `git status` clean. **Refiner pass required** (it writes user files and copies a binary).

**Phase 5: hardening**
- **S5.1 PORT cases**, plus whatever the Windows CI shakes out.
- **S5.2 Break-it cook** (worktree, does not commit). Targets prune and store paths (symlinks, a `..` stream, store not writable), the 16 KiB boundary with multibyte text, frontmatter injection, AGENTS.md marker damage, the accepted prune race (a stream dir swapped for a symlink between check and unlink; closing it needs openat/unlinkat), citation tokenizer edge cases, huge or binary files. Each finding becomes a case and a brief.
- **S5.3 Performance.** In reference-ui: `hyperfine -w3 'memex info' 'memex search namer' 'memex show packages/reference-rs'`. The target is under 200 ms each. Profile first; consult the oracle before adding the §9.4 cache.
- **S5.4 Whole-crate refiner pass and coverage audit.** A scout maps every design.md sentence to case ids and reports gaps as Observed.

**Phase 6: integration**
- **S6.1 Claude Code plugin.** Files: `.claude-plugin/marketplace.json` and `plugins/memex/{.claude-plugin/plugin.json, skills/memex/SKILL.md, hooks/hooks.json, hooks/start.sh}`. Built exactly to design §6 and Spike 2. Done: run the hook with sample hook JSON on stdin; it prints `info` (or the install hint when PATH has no memex).
- **S6.2 reference-ui wiring.** **User checkpoint first.**
  - Run `memex init`.
  - Replace the prototype adapters (`rs-cases`, `cases`→`neo-cases`, `doom`, `flames`, `perf`, `components`, `surfaces`) with manifests.
  - Skills that say "write report to <path>" become `memex log <stream>`.
  - Retire `.agents/memex`.
  - Done: `memex info` shows every collection, no warnings, and counts match the prototype's.
- **S6.3 Platform wiring.** Same as S6.2; the user gives the path.

**Phase 7: Playtest 2 (the success gate; nothing ships before it)**
- **A. Re-run playtest 1 verbatim** (reference-ui atomic namer, Haiku scout, memex first). Pass when:
  - comprehension needed no code reads beyond `rg` symbol lookups;
  - every NAME station is mapped, or is reported uncited via `<-0`;
  - none of the five playtest-1 frictions recur;
  - there were no memex errors to work around.
- **B. Transcript tally** in reference-ui and Platform after about a working week of normal use. Count memex calls against rg/grep calls, and handoffs by ref. Baseline: 6 memex calls vs ~2,000 greps. Pass when memex calls are at least 1 per 10 greps and at least 80% of handoffs go by ref. The user sets the thresholds before the run.
- **C. Friction triage.** Each friction is logged to `playtest`.
  - A bug becomes a slice.
  - A feature needs a user decision (the oracle only if it's a real fork) and a design.md edit before any code.

**Phase 8: release** (after the gate).
- **User prerequisites:**
  - the org, and a public repo;
  - the npm scope (or the fallback);
  - the `homebrew-tap` repo;
  - secrets `HOMEBREW_TAP_TOKEN` and `NPM_TOKEN`;
  - a crates.io token.
- **S8.1 cargo-dist.** `dist init` with:
  - installers: shell, powershell, homebrew, npm;
  - targets: `aarch64`/`x86_64-apple-darwin`, `x86_64`/`aarch64-unknown-linux-musl` (static), `x86_64-pc-windows-msvc`;
  - the generated `release.yml`.

  Then apply the names table to design §6.1 and README.
- **S8.2 RC dry run.**
  - `dist plan` and `dist build` locally, then tag `v0.1.0-rc.1`.
  - Add `install-smoke.yml` on 3 OSes. It installs via the shell/ps1 installer and via `npx <pkg>@rc init` in a temp git repo, then checks that `memex info` exits 0 and the self-install landed.
- **S8.3 Release verdict.** The oracle gives a verdict on behaviour vs design, installer paths and Windows self-install. Then the **user approves**: repo public, tag `v0.1.0`, `cargo publish`, plugin marketplace announced.

**Phase 9: site** (may start during the playtest window).
- `site/index.html` and `site/style.css`; static, no build step, light and dark themes, works at phone width.
- **Story first, from STORY.md:** 1945, the camera and the trail, now, then memex. The memex section covers the intent layer, the five verbs and `npx <pkg> init`.
- Our own retelling. Quotes are short and attributed.
- `.github/workflows/pages.yml` deploys on pushes to main that touch `site/`.
- The orchestrator reviews it in the browser. The user judges the copy against STORY.md's bar ("make people feel what we felt").

## 7. Refiner and oracle
- **Refiner passes, and only these:** S0.2, S1.3, S2.0 (light), S3.3, S4.2, S5.4.
- **Oracle, and only for these:**
  - Spike 1 outcome (c);
  - the cache decision in S5.3;
  - a feature request from Playtest 2 that is a real fork;
  - the release verdict in S8.3.

  Everything else is the orchestrator's call.

## 8. Brief template (paste and fill)
```
GOAL    <one sentence>
READ    design.md §<x>; PLAN.md §2 (+ §1 fixes <n>); `memex show <paths>` output (from S2.0)
FILES   <exact list>; tests/cases/<family>/ (only the ids below)
CASES   write CASES.md entries for: <topics/ids>; then fixtures; bless only these ids
RULES   headers cite the ids they implement; touch nothing outside FILES; design silent → stop and report
DONE    DONE(<filter>) + <slice extras>
REPORT  memex log impl --agent cook-<slice> --tag <family> <<'EOF' … EOF → return `report: <ref>` + DONE tail
```
- **Escalation.** A Haiku cook that fails twice after a SendMessage revision: split the slice, or re-send it with `model: "sonnet"`.
- **Worktrees.** Remove each worktree and its branch as soon as its slice lands.

## 9. Checkpoints that need the user
1. Approve the §1 fixes.
2. Approve Spike 2's temporary hook.
3. Create the private GitHub repo (end of Phase 1).
4. Approve touching reference-ui and Platform (S6.2, S6.3).
5. Confirm the Playtest 2 thresholds.
6. Finish the release prerequisites and approve going public (Phase 8).
7. Sign off the site copy.

## 10. Out of scope
- Everything in design §7.
- A `check` verb: revisit only if Playtest 2 shows agents computing orphans by hand.
- The cache: S5.3 decides.
- Non-English stemming.
- Lenient case-id grammars.

## 11. Tracking
- [x] §1 fixes approved and folded into design.md
- [x] S0.1
- [x] S0.2
- [x] S0.3
- [x] S0.4
- [x] P0 cross-platform audit (2026-10-14; rulings in design §3.4/§4.4; fixes in P1)
- [x] Q0 quality gates (user request 2026-10-13: ≤365 lines/file, headers, READMEs, clippy complexity; tests/quality.rs)
- [x] Spike 1 (confirmed at G1: S1.2 cook's worktree log landed in main and survived worktree removal)
- [x] Spike 2
- [x] S1.1 (G1)
- [x] S1.2
- [x] S1.3 (refined d66f8c5)
- [x] S2.0 (+ S2.0b fixes running)
- [x] P1 portability fixes
- [x] S2.1
- [x] S2.2
- [x] S2.3
- [x] S2.4
- [x] S2.5 (+ S2.6 live MANK cases)
- [x] S3.1
- [x] S3.2
- [x] S3.3 (refined 4c6ab19)
- [x] S3.4 (selfhost gate live)
- [x] S4.1
- [x] S4.2 (self-init: all kept; refiner pass running)
- [ ] S5.1
- [ ] S5.2
- [ ] S5.3
- [ ] S5.4
- [ ] S6.1
- [ ] S6.2
- [ ] S6.3
- [ ] Playtest 2: A
- [ ] Playtest 2: B
- [ ] S8.1
- [ ] S8.2
- [ ] S8.3
- [ ] Site

## 12. Tidy backlog
Cleared by T1 (2026-10-14): header first sentences of 100 chars or fewer, gated; one BOM-dropping text reader; listing-based `.memex` discovery; one root discovery; header prologue and license rules; `preserve_order` JSON fields; collection warnings; stale comments.
Open:
- info reads manifests a second time for counts and JSON warnings: use the corpus (corpus.rs is now 338 lines).
- Out-of-root `show` sections print `(none)` for cites and cited by; needs absolute refs decided first.
- Perf leftovers from the S3.3 refiner: `tokenize::tokens` allocates a String per token (about 14ms); `Git::probe` runs `git rev-parse` twice (about 15ms). S5.3 decides.
