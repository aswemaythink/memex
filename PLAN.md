# memex: build plan

Why: [STORY.md](STORY.md). Bush's trail against his camera: authored context compression.
What: [design.md](design.md) is authoritative on behaviour. This file is how memex gets built, and it replaces plan.md (deleted; the old one is in git history once committed).
Status: blessed by the oracle 2026-10-13. Phases 0-5 done and S6.1 built (2026-10-14); waiting on user checkpoints 3, 4, 5, 6 and 7. Track progress in §11.

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
- **`chmod`** (added 2026-10-14, S5.4): `[{"path": "repo/x", "mode": "000"}]`, octal, sandbox-relative, applied after `setup` and before the steps. It is unix-only, so a case using it must say `"platforms": ["unix"]` (conformity rule 4). The runner restores 0o700/0o600 before deleting the sandbox, and skips such cases when running as root.

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
- `site/index.html`: one static page, no build step, Tailwind from a pinned CDN, light and dark themes, works at phone width.
- **Reworked by the user 2026-10-16: docs first, story last.** Intro, install (`npx <pkg> init` first, the other routes in a fold), quick start, the intent layer and collections, context compression, patterns, the verbs, principles, then the story from STORY.md in full at the foot.
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
RULES   headers state the file's purpose; touch nothing outside FILES; design silent → stop and report
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
- [x] S4.2 (self-init: all kept; refined 76addfe)
- [x] S5.1
- [x] S5.2 (28 findings, 1 HIGH; fixed in F1-F4)
- [x] S5.3 (reference-ui info 94, search 138, show 117 ms; no cache)
- [x] S5.4 (whole-crate refine; coverage audit closed)
- [x] S6.1 (enabling it for this repo needs user approval)
- [x] Perf round (2026-10-15, oracle review:20261014-233020): P-LOAD, P-IDX, P-CITE(+2), P-GIT, P-GIT2, P-NOFAM landed; P-FUSE discarded; no cache. Stress at load 3.7-7.5, 28/28: reference-ui info 52.3 ms, search 68.9, show 56.2 (new budgets 90/115/95); 100k info 994, search 1044, show 1070 ms (budgets 1300/1600/1300); 50k prune 2.42 s. Start of day (70cafdf): 103.6 / 152.7 / 129.6 ms.
- [x] Hardening round 2 (2026-10-14): quiet-machine stress at bf3680f, load 3.9-7.5, 28/28 pass. reference-ui medians: info 79.4 ms (budget 120), search button 108.7 ms (160), show packages 87.9 ms (130); before the round (70cafdf, load 7.5): 103.6 / 152.7 / 129.6. scale_100k: info 1131, search 1259 (budget 2000), show 1189 ms (1500); 50k-entry prune 2.47 s (5 s).
- [x] §15 the graph and its dimensions (2026-10-16): S15.0 frontmatter on every CASES.md (10ee3dd); S15.G citation gates dropped (e90fd4f); S15.1 frontmatter collections reader (13d8924); S15.2 cases becomes an ordinary collection, 110 goldens re-blessed for the info/JSON/name-list lines only (81bd533); S15.3 the markdown manifest kind is gone, MAN-MD-01..15 retired (6fdb906); S15.4 the design §6 snippet everywhere (73697f0); refiner on S15.1/S15.2 (31f1ca0, 4382a61: README read in one open, README one-liner skips frontmatter); tidy (839fc4c). Rulings R11-R22 in §13. BENCH every slice ≤1.05 against its parent (interleaved, load 5-18); reference-ui IDENT where output was meant to stay.
- [x] Onboarding playtest 3 (2026-10-16, cold Haiku, README only, toy Python repo): pass, 0 errors; README gaps fixed (6dcd41d). playtest:20261016-010839-onboarding-playtest-cold-haiku-readme-only-toy-python-repo-p
- [x] 0.1 verdict slices (Fable 5.1, 2026-10-16): R24 a Markdown child lists its title (ac547e2, SHOW-MDCHILD-01..04); R25 the snippet says where a test id goes (5db5b68, c9ad767).
- [x] Frontmatter fields cite (user-approved R28, 2026-10-16): d36f4b6, CITE-FIELD-01..09.
- [x] §14 `memex check` (user-approved 2026-10-16, design §3.5d): C1 verb and intent-layer kinds (7c8f698, 09ec334); line counter (9108592); C2 length and thin, R29 readmes for code folders, R30 thin not a finding (94b95ff); docs and snippet (052577a); selfhost gate (c3805ff); refiner: head facts from the corpus read, per-doc cites, flat cut heading, binary files skipped (92ec5e2, a761eb5); CHK-* 50+ cases. check on reference-ui 280 → 147 ms and on the 100k tree 2.7 → 1.55 s after the refiner (load ~20; budgets 150 ms / 2 s in tests/stress).
- [ ] S6.2
- [ ] S6.3
- [x] Playtest 2: A (2026-10-16, scratch clone of reference-ui wired the 0.1 way, cold Haiku scout, playtest 1's prompt): comprehension pass (46 memex calls, 1 rg); frictions F1-F5 in playtest:20261016-010445-playtest-2a-reference-ui-atomic-namer-cold-haiku-scout-memex
- [ ] Playtest 2: B
- [x] S8.1 (2026-10-15, cargo-dist 0.32.0; publish dry run 100 files / 181.6 KiB; musl static smoke ok. Placeholders: org aswemaythink, npm @aswemaythink, tap aswemaythink/homebrew-tap. Open: installer files are named agent-memex-installer.* (from the crate name), so design §6.1 and README follow once names are final; an rc tag needs version 0.1.0-rc.1 committed; prerelease tags skip brew/npm unless publish-prereleases is turned on.)
- [ ] S8.2
- [ ] S8.3
- [ ] Site (drafted 859b035; awaiting user copy sign-off; deploy held until release)

## 12. Tidy backlog
Cleared by T1 (2026-10-14): header first sentences of 100 chars or fewer, gated; one BOM-dropping text reader; listing-based `.memex` discovery; one root discovery; header prologue and license rules; `preserve_order` JSON fields; collection warnings; stale comments.
Cleared by F1-F4, G-A to G-C and S5.3: init PATH rule; `.memex` inner names by listing; 0644 file modes; plugin hook cwd; §6.1 plugin id; the stale cli README exit-3 note; perf leftovers.
Cleared by 28f100a, 7653f9e, 693d754, d8e06a0 and the 2026-10-14 hardening pass: the nine S5.4 items (manifests loaded once, collections probed by listing, init's writer, one fence scanner, Windows PATH scrub, `$MEMEX_NOW` validated once, INFO-CAP-02 pinned by a unit test as design §4.4 allows, CLI-EXIT-04 live, out-of-root `show` sections, now SHOW-OTHER-01).
Open: none.

## 13. Rulings to review (made where design.md was silent)
Each line is a design.md commit since S0.1; `git show <hash> -- design.md` shows the exact wording. Rulings made from here on are added to this list.
- `f289152` design: §3.4 punctuation; PLAN: Phase 0 tracked
- `bebab8b` design: portability rulings from the cross-platform audit
- `b1ff12e` design: `.` is always the root; children omit the folder's README
- `c64e6a3` design: markup prologues are pragmas; license block ends at a blank line
- `a64bab3` design: limits hint only when a list is cut; PLAN tracking
- `47ba8f8` design: init ignores npx's node_modules\/.bin shim when checking PATH
- `fe186e8` S5.1: PORT cases; CRLF text reads as LF (cherry-picked df1ace5)
- `cb6e671` design: break-it rulings (S5.2)
- `6429cac` design: lone CR in a frontmatter value exits 2; PLAN backlog
- `e4a1323` design: repeated refs print once; a requested unreadable file is an io error
- `ebbe4a0` design: settle the S5.4 audit's case/design contradictions (C1-C8)
- `7829dd3` design: one fence definition everywhere; PLAN S5.4 done
- `1e58663` design: fence closer rule, as fence.rs implements it
- (hardening, 2026-10-14) design: R2 closed stdout is silent, exit 0 (§3.6); R3 a requested folder that can't be listed is that ref's io error (§3.3, extends e4a1323); R4 vanished log entries are skipped silently, writes re-create a removed stream folder (§4.3); R7 INFO-CAP-02 joins TTY as a unit-test-pinned exception; races and pipes are pinned by tests/stress (§4.4).
- (hardening, user-approved 2026-10-14) design: R1 inside git, `.memex/` discovery stops at the toplevel (§4.1); R5 names with control characters are skipped like non-UTF-8 names (§4.4); R6 another root's cites/cited-by print a pointer line, JSON null (§3.1). Orchestrator: a closed stdout keeps the exit code the verb would have had (§3.6, refines R2); a path inside an unlistable folder is `not found` (§3.3); prune's DirectoryNotEmpty on a folder a writer just landed in is silent (§4.3, F1); D2 keeps resolve::normalize, since on Windows it parses typed refs with `\` and prefixes that refs::join must not.
- (hardening round 2, 2026-10-14) design, accepted limits (orchestrator): MED-03 stream-folder symlink swap needs openat (§4.3); L11 an agent frontmatter block ends at its first `---` (§3.4); a far-future `$MEMEX_NOW` prunes the whole store (§4.3); `GIT_WORK_TREE`/`core.worktree` can let reads find a `.memex/` above that work tree (§4.1). User-approved behaviour changes: fuzzy and prefix lengths and edits count chars, not bytes (§3.2); a skipped name (control char, non-UTF-8) gets no unreadable warning (§4.2); init's taken-name race says "another process created it while init ran", exit 3 (§3.5b).
- (hardening round 2, user-approved 2026-10-14) design: messages name the next step (§3.5c); zero hits print a stderr hint, stdout empty, exit 0 (§3.2); words are Unicode letter/digit runs (§3.2); init outside git targets the discovered root, so no shadowing store; a hand-pasted snippet without markers is warned about and left (§3.5b); `log` in a linked worktree with no git exits 3 (§4.4 git); one default title rule for every manifest kind, no sentence cut (§4.2); unreadable `merge` files warn; a symlinked manifest root leading outside the repo is skipped with a warning (§4.2). Orchestrator: write retries are bounded at 64 (§4.3, B1); init's race message also names a case-only clash, which case-insensitive filesystems give (§3.5b); a main-checkout entry has no `worktree:` field (conc_worktrees asserts it; root.rs Place.worktree).
- (round 2, cook-init rulings) a hand-pasted snippet reports `kept`, like damaged markers; when an unclosed fence and a pasted snippet both hold, the unclosed-fence warning wins; a bare repo takes init's outside-git rule (unpinned). trim_eol moved unchanged to text.rs beside line_of (they differ: one strips all trailing CR/LF); merging them is left open.
- (round 2, C3) the no-panic gate skips test-only files by an explicit TEST_ONLY list in tests/quality.rs (today src/collections/testing.rs), not by parsing `#[cfg(test)] mod` lines; `.unwrap()` is gated too (zero non-test uses at f6d93d2). Clippy's expect_used/panic lints stay off: tests/ helpers would need #[allow].
- (round 2, orchestrator, refines the user-approved NEST rule after the refiner found $HOME adoption) init outside git that adopts a store above the cwd says so on its first line and names `mkdir .memex && memex init` to start one here (§3.5b).
- (round 2, cook-unicode rulings) a word character is `char::is_alphanumeric` (so ½ and Ⅻ count); combining marks split words (no Unicode normalisation); capital sigma lowercases as `str::to_lowercase` gives; a dot before a Unicode letter makes an id or path token, as before for ASCII.
- (round 2, cook-titles rulings) a title field's text is not sentence-cut either; a manifest root that cannot be canonicalised is left to the walk; 'the repo root' for the outside-root check is the memex root. MANK-MERGE-01 and MANK-ROOTS-01 are unix only.
- (round 2, B5) without git, a linked worktree is recognised by the nearest `.git` entry at or above the cwd being a file, so a sub-folder of a worktree with its own `.memex/` is refused too; the refusal's JSON code is `io` (§3.6 exit-3 environment error).
- (round 2, cook-messages rulings) the unknown-collection list includes `logs`; a roster with no words and no filters prints no zero-hit hint; a roster narrowed by filters uses the filters form; the hint goes through error::warn; filter values fold with str::to_lowercase, field names stay ASCII-case-insensitive. CLI-HINT-12 (init race) stays planned: it needs a concurrent writer.
- (round 2, B5 notes) without git a submodule's `.git` file also counts as a linked worktree for log's refusal; a symlinked `.git` is not treated as a file; info without git still names the worktree's own store as log_store (DISC-NOGIT-02).
- (round 2, P2) the index's term shards split by DefaultHasher with fixed keys (work split only, output order unchanged).
- (round 2, orchestrator, after onboarding playtest 2) info's `to` line shows at zero, as §3.5 already said (code diverged); usage and info's `use` line show the heredoc form, not `< body`; two more messages name their next step (§3.5c); §3.2's fuzzy example is naïve/naive (café is under the 5-character floor).
- (round 2) lists sort by byte (code point) order, never locale (§3.6), pinned by SHOW-ORDER-01.
- (round 2, cook-polish) the to line at zero also shows with no store (INFO-NOROOT-01), the literal §3.5 rule; the Windows-only INIT-INSTALL-02, INIT-PATH-04/05 goldens were hand-edited for the heredoc use line and await a Windows run.
- (user-approved 2026-10-14, oracle review:20261014-212828) git is an enhancement: the toplevel is found by listing, always (U-G1); a failing git is treated like a missing one (U-G2); without git the store and HEAD come from the layout files (.git, gitdir:, commondir, HEAD only) (§4.1, §4.4). New output: show's gap legend (§3.3), search group headings name their markers, a single-filter zero hit says 'try another value' (§3.2). info's '0 headers' gets no hint (oracle: standing noise).
- (round 2, H2 rulings) the fuzzy legend lists each distinct variant once, alphabetically; a single filter is exactly one term and no words; the roster legend comes from the lines shown, so --limit 0 prints none (SRCH-LEGEND-01).
- (round 2, G1 rulings) a folder whose listing of `.git` fails counts as listing nothing, so the walk goes on past it; DISC-TOP-04's 'empty' .git folder holds a placeholder (git can't store an empty folder); with an empty .git folder in a subfolder, reads name the subfolder as root while git still places the store in the outer repo (accepted edge, not pinned by a log step); DISC-TOP-03 stands in for a failing git with GIT_DIR=/nonexistent.
- (round 2, G2 rulings, orchestrator-approved) in a repo with no commit, branch comes from HEAD (`main`) with or without git (LOG-PRUNE-13..17 re-blessed); a bare repo whose folder is named .git is placed as a main checkout without git (accepted); reftable cases wait for a git-version gate (2.39 lacks --ref-format). Reads still spawn two git calls (info's store line): a target for the performance round.
- (round 2, refiner on G1/G2) layout files are read only as regular files ≤16 KiB (a FIFO .git hung log); a commondir must name a folder; the store maps to a main checkout only when the toplevel's own .git is a file, so git's answer and the listing root never mix (DISC-TOP-07/08).
- (user-approved 2026-10-15, perf round) a file with no comment family is never opened, so it gives no unreadable warning (U-P1, §4.2); git is asked for the store only when the toplevel's .git is a file (U-P2, §4.4). Orchestrator: index runs and the corpus are never freed (short-lived process; P-IDX, P-LOAD); no cache (oracle review:20261014-233020: reference-ui ~63-80 ms cold, under the §9.4 200 ms trigger).
- (round 2 perf, orchestrator) a no-family file is never opened by the walk, but a merge file is read by its manifest and still warns; warning order follows the order they arise (MANK-MERGE-01 re-blessed for order only).
- (round 2 perf, orchestrator, P-GIT2) with no toplevel, git is asked for the store only when a .memex/ is in reach, keeping §4.1 item 3's bare-repo store; a .git folder never asks git.
- (round 2 perf) P-FUSE (reads inside the walker callback) measured and discarded: byte-identical but no consistent gain at 8 threads (reference-ui 0.96-1.13, tree100k 0.99-1.03); patch kept in the session scratchpad only.
- (PLAN §15 rulings R1-R10, oracle, 2026-10-15; U1-U3 and R1 user-approved 2026-10-15) R1 README.md never joins a collection (the graph's own; a `memex` key in its frontmatter is ignored with a warning that tells the user to move the content into its own file), since a stem id would make every README `README`; R2 collection names use the stream grammar `[a-z0-9][a-z0-9-]{0,39}`, and any other value warns and the file is skipped; R3 the key counts only within the first 8 KiB (the header limit, §2.1); R4 entries do not inherit the file's frontmatter, and an entry index is not itself a doc; R5 a manifest and frontmatter files with one name form one collection, and `info` shows the manifest's kind, description and owner; R6 frontmatter `fields` omit `memex`, and `id` stays a field; R7 JSON kind `markdown` for frontmatter collections; R8 `kind: markdown` manifests are skipped with a warning that names the fix; R9 duplicate ids warn per collection (manifest docs now warn too), each path named once; R10 `logs` as a collection name warns like the graph names.
- (§15, S15.1, orchestrator, 2026-10-16) R11 the `memex` key counts only when its whole line lies within the first 8 KiB (COLL-FRONT-06; R3 said only "within"); R12 a label shared by several docs (duplicate ids) stays one search hit: the duplicate is an authoring error that `info` warns about and `show` reports as ambiguous; R13 an unreadable Markdown merge file now warns when the walk meets it, so MANK-MERGE-01's warnings reorder (design §4.2 "in the order they arise"); R14 a manifest named `readmes`, `headers` or `logs` is still rejected (the §4.2 table), a manifest named `cases` loads, and a frontmatter collection with a graph, `logs` or stream name loads with the §4.2 Names warning.
- (§15, S15.2, cook rulings accepted by the orchestrator, 2026-10-16) R15 a frontmatter collection named `logs` gets the stream form of the Names warning (`--in logs means the stream`), since the graph form would be false; R16 json and folder manifest docs have no file of their own, so duplicate-id and shared-name warnings name the manifest file; R17 a collection sharing a graph or log name is its own search group, so two headings (and JSON `collection` fields) can read the same name; R18 `show logs:<id>` takes a collection doc with that id first; R19 shared-name warnings print on every verb that loads the corpus, like manifest warnings, while duplicate-id warnings stay `info`-only as design says.
- (§15, refiner on S15.1/S15.2, orchestrator, 2026-10-16) R20 a closed frontmatter block atop a README is not part of its one-liner text: the one-liner is the first paragraph after the H1, as design §2 already says (COLL-REFINE-03); R21 the invalid-name warning prints a list value as written, `'memex: [a, b]'`; R22 a `.md` marker or merge file of a folder manifest is still read by the manifest after the walk's 8 KiB probe (reusing the probe would hold every `.md` in memory); accepted for 0.1.
- (playtest 2A fix, orchestrator, 2026-10-16) R23 a manifest doc's folder for its relative citations (§2.3) is the folder holding its marker (folder kind) or its JSON file's folder (json kind); its JSON `path` stays null (MANK-CITE-01/02, 2071dda). R11, R12, R14-R16, R20 and R23 are folded into design.md §2, §2.2 and §4.2 (this commit).
- (Fable 5.1 final verdict, 2026-10-16; design contradicted itself: §2.2 gives a Markdown doc a title, §3.3 listed it `(no header)` with advice that cannot be followed) R24 a Markdown file's child line in `show <folder>` shows its title (the §4.2 title rule after any frontmatter), no placeholder and no legend line; an empty one shows its name alone; JSON `title` carries the same text (design §2.1 Limits, §3.3 Folder). R25 the snippet's cases lines name where an id goes (in the test's name where the language allows, else in a comment on it) and point at the collection `memex info` names when there is no `cases` (design §2 convention 3, §2.2 Evidence, §5, §6), after playtests 2A (F1) and onboarding (O2).
- (R24 cook rulings, accepted by the orchestrator, 2026-10-16) R26 a Markdown child's title is looked for in the first 8 KiB, whole lines only; an unreadable `.md` child shows its name alone and keeps its warning; an empty one has JSON `title` "". SHOW-GAP-01's claim now pins a title row; SHOW-DIR-01 still pins the `(no header)` legend. Folded into design §3.3.
- (user 2026-10-16: "ship with check"; orchestrator took §14's recommendations) R27 `check` (design §3.5d): name `check`; exit 0 whatever it finds (1 only for a path that names nothing); kinds in a fixed order headers, readmes, length (over 500 code lines), one-liners (over 100 chars), thin (80+ lines, lowest header ratio, no threshold), cites (unresolved; logs not checked), ids (duplicates); `--limit` default 20 per kind; a glob is an argument holding `*`, `?` or `[`, matched with the manifest `include` rules; manifest docs are in scope by their folder; the snippet gains one `check` line. Q3 (id-prefix search) dropped: patterns come from metadata (`field:value` on frontmatter keys, entry `group`).
- (user-approved 2026-10-16) R28 frontmatter field values of a collection doc cite like its body (all keys but `memex`; list items one by one; before the body in `cites` order); a log's frontmatter is not scanned (design §2.3).
- (check C1 review, orchestrator, 2026-10-16) R29 `readmes` flags a folder only when its subtree holds a file with a comment family (data-only folders such as case folders are not units; the literal rule flagged 668 here, 2773 in reference-ui); R30 `thin` is a pointer, not a finding (else `check` could never say `nothing to fix`); R31 C1's design-silent rulings accepted (impl:20261016-090300-check-c1-the-verb-its-scope-and-the-intent-layer-kinds §Design-silent 1-13: empty = zero bytes; a path resolves as show resolves it, so a unique basename counts; labels and bare ids are not scopes; counts of one are singular; ids in scope when any doc is; logs never checked); R32 the snippet's last line runs `memex check <paths>` (design §6).
- (check C2 review, orchestrator, 2026-10-16) R33 `thin` is listed in JSON `kinds` but not counted in `findings`; `nothing to fix` prints right after the scope line and `thin` may still follow (§3.6); a cut kind's heading is `(showing N of M; --limit M for all; <what>)`, not nested parentheses (§3.5d); `id` is scanned like any frontmatter field (R28's literal reading; a doc whose id equals another collection's id cites it, as the body would); json-manifest and JSON-marker fields do not cite (not frontmatter). selfhost gates `memex check` on this repo: nothing but `thin` and case claims that quote fixture refs.
- (refiner on check, 2026-10-16) R34 `cites` reads each doc for itself: two docs sharing a label each report their unresolved refs, and a (label, ref) pair is listed once (CHK-REFINE-02); R35 a binary comment-family file is skipped by `headers`, `length` and `thin`, as §2.1 skips binary files (orchestrator ruling; CHK-REFINE-04).
- (show --comments, orchestrator, accepted by the coordinator 2026-10-17; design §3.3 Comments) R36 the comments list sits after the body and its gap line, before `cites`, also with `--up 0`; R37 a row is `  L<n>` then the block's lines, markers removed as the header prints them, indented four spaces; R38 a block runs as the header does (§2.1): blank lines inside it do not end it, the first line that is neither comment nor blank does; R39 the blocks listed are those after the header, or after the lines §2.1 skips (shebang, pragmas, license block) when there is none; a docstring other than the module docstring is not a comment; R40 a list cut by `--limit` is headed `comments  (showing N of M; --limit M for all)`; the no-header list shows 3, or `--limit` if lower, and is left out when the file has no other comment block; with `--comments` an empty list prints `(none)`; R41 a file with no comment family (Markdown, unknown extension) or a binary file has no comments: `--comments` is ignored and nothing falls back; R42 JSON `comments` is present exactly when the text prints the list, `line` counts from 1, `text` is the block as printed, unindented; R43 help: usage gains `[--comments]`, a line `--comments  also print the file's other comment blocks (a file with no header prints its first 3)`, and the `--limit` line ends `(default 50; comments 20)` (help text changes for existing help goldens).
- (show --comments implementation, orchestrator, 2026-10-17) R44 the no-header heading says `--comments for all` even when `--limit` cut the list (SHOW-COMMENTS-06); a file cut at 1 MiB counts M over the blocks read, and JSON `cut` means the file was cut there (SHOW-COMMENTS-09); a blank line inside a block prints empty (SHOW-COMMENTS-01); a Python docstring before any statement but after a `#` header is the module docstring and is listed (cook ruling). Re-blessed: HDR-CAP-01 and HDR-SKIP-07 (block.rs) gain the no-header list, as design §3.3 now says.
- (show --comments, cook rulings accepted by the orchestrator, 2026-10-17) R45 a triple-quoted string that is not a docstring is skipped whole, so its lines never read as comments; a read cut at 1 MiB drops its unfinished last line, and with `--comments` and no block read prints `(none)` then the cut row; a run that starts with a bare marker lists at that line; pragma-like lines after the header (`//go:`, `{-#`) are listed as blocks.
- (log size cap, orchestrator orch-logcap, 2026-10-17; user input: logs are ephemeral, 10 MB per repo, set in MB in .memex/config.json, pruned on every log; design §4.1 item 4, §4.3 Cleanup) L1 `max_logs_mb` is a positive whole number of MiB (1,048,576 bytes), default 10, capping the whole store (all streams, the main checkout's store); L2 only regular files named `YYYYMMDD-HHMMSS-<slug>.md` directly in a non-symlink, non-hidden stream folder count, by file length in bytes, and only they are deleted; L3 after the write, keep_logs (if set) prunes first, then the oldest entries go until the total is at or under the cap; oldest is by filename stamp, ties by entry name then stream name, in byte order; L4 the new entry counts but is never deleted; L5 a refused delete warns and its bytes still count (the next oldest goes); a vanished entry counts as gone; L6 `keep_logs` is optional and off unless set (no 3d default); `forever` stays valid; L7 a file that can't be read, isn't JSON, isn't an object or has an unknown key prunes nothing at all; an invalid value disables only its own limit, warning `invalid max_logs_mb <v>: not pruning by size` / `invalid keep_logs <v>: not pruning by age` (strings in single quotes, other values as JSON text); 0, negatives, fractions, strings and caps too large to count in bytes are invalid; L8 init writes `{"max_logs_mb": 10}`; an existing config is never rewritten; L9 design §1's "useful for a couple of working days" stays (true in spirit); §9 open question 6 is settled by the user's input. Re-blessed: init/discovery goldens that show init's config.json, LOG-PRUNE-01, -07, -09, DISC-CONFIG-02 (serde's unknown-key message names both keys); age cases that relied on the 3d default now set keep_logs 3d in their fixtures.

## 14. `memex check` (planned 2026-10-15; approved by the user 2026-10-16: 0.1 ships with it; design.md §3.5d)

Answers design §9.3 ("a `check` verb for drift?"). The user's motivation: encode what makes memex work well, so the intent layer stays granular enough to build good topologies from it. Small files keep the header-to-code ratio tight; units without intent can't be cited.

**Goal.** One verb that reports how healthy the intent layer is for a file, a folder, a glob or the whole root, so an agent can see where intent is thin and fix it. It reports facts memex already knows, plus one plain-text measure (line count).

**Shape (to approve in design.md first).**
- `memex check [<path|glob>...] [--limit N] [--json]`. With no argument it checks the root (discovery as §4.1). A glob matches walked labels, the same matcher as manifest `include`.
- Text output is grouped by finding kind, in a fixed order, each `showing N of M` (§1 bounded output):
  ```
  check  src/  148 files · 12 folders
  headers    3 missing      src/a.rs · src/b.rs · src/c.rs
  readmes    1 missing      src/util/
  length     1 over 500     src/big.rs 612 code lines
  one-liners 1 over 100     src/y.rs 131
  thin       5 lowest       src/big.rs 1:306 · src/q.rs 1:140 · …
  cites      2 unresolved   src/z.rs → NAME-404 · docs/README.md → ../gone.md
  ids        1 duplicate    cases:NAME-003 (2 places)
  ```
  The `(no …)` legends of §3.3 Gaps are reused as each kind's fix line. JSON is `{scope, counts, findings:[{kind, ref, detail}]}`.
- **Header ratio (reported, not enforced).** For each file with a header: header lines against total lines. `check <file>` prints it; the `thin` kind lists the lowest ratios among files over a small floor (say 80 lines), worst first, with no pass/fail threshold. It is a pointer to where intent is thin, not a rule.
- **Why lines stand in for complexity.** A file kept under 365 lines trends toward simple code, and growth pushes the complex part into its own file with its own header, which is what memex wants. So `check` measures lines and header ratio, never parsed complexity.
- **One verb, every kind.** All findings roll into this single check; there are no sub-verbs or per-kind flags.
- Thresholds are fixed defaults in v1: a file flags at **over 500 code lines** (user, 2026-10-15). Code lines are total lines minus the leading header block and any license block (§2.1); blank lines and inline comments still count. A header can easily run 50 lines, and counting it would reward thinner intent, the opposite of memex's goal (user, 2026-10-15). `thin` still uses total lines against header lines, so a generous header only helps. memex's own 365-line bar is this repo's house style, too opinionated for everyone; 500 is where a file clearly needs splitting. One-liners flag over 100 chars. Config keys only if a playtest asks for them, since §4.1 item 4 keeps config.json to one key.

**Not a kind (user, 2026-10-15): uncited cases.** Hand-kept links from code to cases are maintenance that rots; agents infer them. `check` never asks for them. Duplicate ids are checked per collection (see §15).

**Out of scope (decided).** Cyclomatic complexity, function length and any other parsed-code metric: they need per-language parsers, which breaks §1 ("memex reads intent, `rg` reads code; never indexes code bodies") and the small-on-purpose surface. Language linters own them; the AGENTS.md snippet may point to them. No auto-fix and no new log behaviour (`memex log quality` already works as a stream).

**Performance.** Line counts need whole-file reads (today memex reads only the first 8 KiB), so only `check` reads past 8 KiB. It counts newlines streaming, in parallel, only for files with a comment family (P-NOFAM's set); a file under 8 KiB already holds its whole text, so it needs no extra read, and never caches. `info`, `search` and `show` must not change (a BENCH gate against main). Budget: reference-ui `memex check` under 150 ms, 100k tree under 2 s.

**Order of work.**
1. Oracle: draft design.md text (§3.x `check`, §3.6 JSON, §9.3 resolved, exit codes) and a CASES family `check` (topics SCOPE, KINDS, LENGTH, LIMIT, JSON, GLOB). The user approves.
2. Cook C1: verb skeleton, scope and glob, and the intent-layer kinds from the existing corpus and cite index (headers, readmes, one-liners, cites, cases, ids). No new reads.
3. Cook C2: line counts (`length`), with its BENCH and the guard that the read verbs stay unchanged. It runs in parallel with C1 if FILES are disjoint (C1 holds the verb and render, C2 a counting module).
4. Refiner on C1+C2 (hot path and bounded output). Then a cold playtest: does an agent in a fresh repo use `check` and fix what it says?
5. Budgets go into tests/stress; README and AGENTS.md snippet lines; PLAN §11.

**Open questions for the user (recommendations first).**
1. Name: `check` (rec.; matches §9.3, short, reads as a tool) or `quality`.
2. Exit code with findings: 0 always (rec. for v1; it's a report, and agents read the text) or a non-zero code for CI gating, which needs a new code in §3.6.
3. Does the AGENTS.md snippet tell agents to run `memex check` before they finish? Rec.: yes, one line, so the habit forms.
4. Settled: `length` counts code lines (header and license blocks excluded). Once `check` exists, tests/quality.rs's 365 gate should use the same count (separate change).
5. `thin` floor and list size: rec. files of 80+ lines, the lowest 10 by header ratio (bounded by `--limit`).

## 15. The graph and its dimensions (approved 2026-10-15; design.md applied 2026-10-16)

A design change from discussion with the user. It replaces cases as a built-in with collections declared in frontmatter, and adds a stat-keyed cache gated on measurement. Sequence it before §14, since `check` builds on it.

**The model.**
- **The graph:** READMEs (one per folder) and headers (one per file), attached to the tree and found by *where* they are. `show <path>` walks it.
- **Dimensions:** collections that cut across the graph and are found by *what* they are (cases, perf notes, decisions, playtests). Bush's trail-blazer patterns, which anyone can start.
- **Citations** tie the two together, both ways. **Logs** are the time dimension, scaffolding that expires (unchanged).

**Rule: a frontmatter line joins a dimension.** Any Markdown file whose frontmatter has `memex: <collection-name>` (user, 2026-10-15) is a doc in that collection, with no manifest and no config. It is local, so an agent can follow it and check it in the file it is editing.
- One file is one doc: id = `id:` field, else the file stem; title = the first heading; the other frontmatter keys are filterable fields (§4.2 markdown rules, without the manifest).
- Many docs per file (OPEN, rec. yes): if the file's column-0 list items start with a backticked id, each item is a doc (today's CASES.md rule, made generic). `CASES.md` with `memex: cases` then keeps working and stops being special.
- Citations unchanged: ids matching the case-id grammar stay citable bare; any doc is citable as `<collection>:<id>`; backlinks as today.
- Search groups: the graph (`readmes`, `headers`), then dimensions alphabetically, then `logs`.

**Names (user, 2026-10-15): warn, never break.** A collection named like the graph (`readmes`, `headers`) or like a log stream still loads. Its docs are indexed, appear in global search under their own group, and resolve in `show`. `--in <name>` keeps meaning the graph or the stream. A warning prints once per run in the §4.2 form, `memex: warning: collection '<name>' (in <path>) shares a name with the graph; --in <name> means the graph — rename it to reach it with --in`, and `info` lists it. A one-doc collection (likely a typo) is something `check` can report.

**What goes.** The built-in `cases` collection; the `markdown` manifest kind (frontmatter covers it); the house rule that every header carries a `Cases:` line and the selfhost gate `every_case_is_cited_directly` (separate repo-level change, user decides; ids and claims stay). **What stays:** manifests of kind `folder` and `json`, for structures that are not Markdown (e.g. reference-ui's `case.json` folders).

**Change detection (gated on measurement; design §9.4).** memex builds everything per run today, so nothing is "rebuilt". The cost is open(), about 13 µs here against 2.5 µs for stat (oracle review:20261014-233020). If frontmatter discovery's extra `.md` opens cost too much, or big monorepos need under ~0.5 s:
- walk and stat everything as now; keep a per-file record in `.memex/cache/` (reserved and gitignored by design) keyed by path, size, mtime and inode, holding the extracted header block, the frontmatter `memex:` value, the one-liner and the citation tokens;
- re-open only files whose stat changed; build the search index per query from the records (after P-IDX it is cheap);
- racy rule as git does it: a file modified within ~2 s of the cache write counts as changed; atomic writes via `atomic::persist_new`; a read writes only when something changed; no CLI change; repos without `.memex/` stay on the cold path;
- not git's index (`git status` is 81 ms here and misses untracked files and non-git repos), and no watcher (a daemon, §1).
Expected: reference-ui `info` about 55 → 25–30 ms; 100k about 1 s → 0.4–0.5 s. Output must stay byte-identical (IDENT).

**Order of work.**
1. Oracle: measure frontmatter discovery on reference-ui, the 100k tree and this repo (how many `.md` files, open cost, warm vs cold). Draft the design.md rewrite (§2 conventions around graph and dimensions, §2.2, §3.2 groups, §4.2) and the case plan (a new `collection` family; migration of the cases family). Rule on whether the cache is needed now, with numbers. The user approves.
2. Cooks: frontmatter collections (discovery, docs, entries rule, name warnings); migrate this repo's CASES.md files (one frontmatter line each); remove the built-in `cases` and the markdown manifest kind. Then a refiner pass.
3. If the oracle rules yes: the stat-keyed cache, with a refiner on correctness (racy edits, concurrent writers) and a BENCH.
4. Then §14 `check` on the new model. Then a cold playtest.

**Approved by the user (2026-10-15).** The model and the `memex:` key; many docs per file via id-led list items; drop the built-in `cases`, this repo's `Cases:` header rule and the selfhost citation gate; the cache is built only on the oracle's numbers.

Design applied in commit "design: the graph and its dimensions (PLAN §15, user-approved)"; U1-U3 and R1 approved by the user 2026-10-15.
- (§15, S15.G, user-approved) the `Cases:` header gate and the direct-citation selfhost gate are gone; citing case ids in headers is optional; existing `//! Cases:` lines stay as ordinary citations.

## 16. After the 0.1 verdict (Fable 5.1, 2026-10-16)

Verdict: approve 0.1, modulo the user-blocked items. The hard pre-tag gates are all the user's: Windows conformance green in CI before `v0.1.0` (design §4.4 promises it; it has never run, and the Windows-only goldens were hand-edited), the names applied (design §1 and §6.1 say `rynwhite`/`aswemaythink (pending)`, Cargo.toml and §0's table say `aswemaythink`; installer files are `agent-memex-installer.*`), S8.2 RC dry run, and site copy sign-off.

Done tonight on the verdict: R24 (a Markdown child lists its title in `show <folder>`) and R25 (the snippet says where a test id goes and points at the collection `info` names), §13.

**Proposal for the user (not built): Q3, id-prefix search.** Playtest 2A's `memex search ATM-NAME` found nothing, because an id-shaped query token matches whole id tokens only (design §3.2, SRCH-TOKEN-01). Proposed rule: an id-shaped query token also matches ids that continue it after a hyphen (`ATM-NAME` → `ATM-NAME-01`…), as a prefix tier ranked below an exact id; SRCH-TOKEN-01 is unchanged (`NAME-001` still does not find `NAME-002`). It changes a stated rule, so it needs your yes and a design §3.2 edit. (The one hit `ATM-NAME` gave, NEO-CSS-11, is correct: that README cites `ATM-NAME-*` literally.)

**Left as is (reviewer's ruling).** Q5: `show` prints ancestor READMEs in full (design §3.3); `--up 0` is the escape; revisit after Playtest 2B. Q6 (superseded by §14, user 2026-10-16: 0.1 ships with `check`): 0.1 ships without `check` (design §7 and the README name it a non-goal today); §14 stays a proposal.
