# cases

This folder is memex's conformance case hub. Every behaviour the CLI must have is pinned by a case here. Each family is a folder, such as `cli/`, that holds its own README.md, a CASES.md claim index, and one folder per case, named by the case id.

## Layout

```
tests/cases/README.md                    this hub
tests/cases/<family>/README.md           what the family covers, its design §, its prefix
tests/cases/<family>/CASES.md            the claim index: one entry per case
tests/cases/<family>/<ID>/case.json      the run spec (required)
tests/cases/<family>/<ID>/expect.txt     the golden transcript (written by bless)
tests/cases/<family>/<ID>/fixture/       the sandbox tree (optional); a path part `dot_x` becomes `.x`
tests/cases/<family>/<ID>/<name>         a stdin file, only when case.json names it in stdin_file
```

## Ids

An id is `PREFIX-TOPIC-NN`: capitals, a dash, capitals, a dash, and two digits, as in `CLI-VERSION-01`. The prefix is the family's, and every entry of a family shares it. The topic is the middle segment, and it must match the `## TOPIC` heading the entry sits under. Ids are unique across the whole tree.

## Entries (CASES.md)

```
---
memex: cases
---
# CLI cases

## VERSION

- `CLI-VERSION-01` `--version` prints `memex` and the crate version, exit 0.
  design §3. Run: memex --version.
```

An entry is a column-0 line that starts with ``- `ID` `` and has a claim after a space. Its detail lines are indented by two spaces and say the scenario and the design §. A claim that contains `(planned)` has no folder yet, so write planned cases first and add the folder when the case is built.

## case.json

```json
{"git":"commit","setup":[{"run":["git","worktree","add","../wt","-b","feat"]}],
 "env":{"MEMEX_AGENT":"cook-2"},
 "steps":[{"args":["log","impl"],"stdin":"Title\nbody\n","cwd":"wt"}],
 "snapshot":["repo/.memex/logs"],"memex_on_path":false,"platforms":["unix"]}
```

Unknown keys are an error. Every field is optional except `steps`, which needs at least one step.

- `git` is `init` or `commit`. It makes repo/ a git repository on branch `main` from the fixture, with no system templates, a fixed identity and the date 2026-10-13T11:00:00Z.
- `setup` is a list of `{"run": [argv]}`. The argv starts with `git` or `memex` (the built binary), and it runs in repo/. A nonzero exit fails the case, with its stderr.
- `steps` each take `args`, then `stdin` or `stdin_file` (not both), `cwd` (relative to the sandbox, repo/ by default) and `env`.
- `env` sets variables for the whole case. A step's `env` overrides it.
- `snapshot` lists sandbox paths whose files are dumped after the last step.
- `memex_on_path` prepends the built binary's folder to PATH.
- `platforms` is a list of `unix` and/or `windows`. A case that excludes this OS is an ignored trial.
- `chmod` is a list of `{"path": "repo/x", "mode": "000"}`: an octal mode of one to four digits for a path inside the sandbox. The modes are set after `setup` and before the steps. Modes exist only on unix, so a case with `chmod` must say `"platforms": ["unix"]`. After the case the runner gives the owner back rwx on each folder that a change touched, and rw on each file, so the sandbox is always removed. Root reads any file whatever its mode, so when the runner is root every case with `chmod` is an ignored trial. Root is detected by making a mode-000 probe file and reading it.
- `fill` is a list of `{"path": "repo/.memex/logs/impl/20261012-120000-big.md", "bytes": 10485000}`: the runner makes each file with that many NUL bytes, as a sparse file where the filesystem allows it, so a case need not commit a large file. The path is relative to the sandbox and uses real names (`.memex`, not `dot_memex`), as `chmod` paths do. The runner makes the parent folders. The files are made after the fixture copy and the git step, and before `setup`, `chmod` and the steps. A path that leaves the sandbox, or a file that already exists, fails the case when it runs.

## Sandbox

Each case gets a fresh temp folder with repo/ and home/. The fixture is copied into it. The environment is the inherited one, minus `MEMEX_*`, `GIT_*`, `CLAUDE*` and `AI_AGENT`, and minus every PATH entry that holds a memex executable. Then the defaults are set: `HOME` and `USERPROFILE` are home/, `XDG_CONFIG_HOME` is home/.config, `LOCALAPPDATA` is home/AppData/Local, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL` is an empty home/.gitconfig, `GIT_CEILING_DIRECTORIES` is the sandbox, `NO_COLOR=1`, `LC_ALL` and `LANG` are `C`, and `MEMEX_NOW=2026-10-13T12:00:00Z`. The case's `env` and then each step's `env` override those.

## Golden format (expect.txt)

Each step renders as a command line, an exit line, and its streams:

```
$ memex log impl --tag show  [cwd wt]
exit 0
stdout
| impl:20261013-120000-title
stderr (empty)
```

Steps are separated by one blank line. Then each snapshot renders as `== snapshot <path>`, then each file as `-- <path>` and its rows, or `(missing)` for a missing path and `(empty)` for a folder with no files. Every output line is a `| ` row. A text that does not end in a newline ends with `\ no newline at end`. Arguments, the `[cwd X]` path and each `[env K=V]` pair are quoted only when they hold anything outside `[A-Za-z0-9._/:@%+=,-]`. Redaction replaces the sandbox path with `$SANDBOX` and the crate version with `$VERSION`, and nothing else. The version is replaced only where no digit or `.` precedes it and no digit or `.digit` follows it, so `10.1.0` stays as it is. The golden is compared byte for byte.

## Conformity rules

Six rules run before any case, and each is its own trial, `conformity::rule1` to `conformity::rule6`. Each message reads `ruleN: <what> at <path>: <how to fix>`.

1. Every entry under `tests/cases/` is `README.md` or a family folder. A family folder holds `README.md`, `CASES.md` and case folders only.
2. Every CASES.md parses. Ids match the pattern, share one prefix per family, the prefix is unique across families, ids are unique, and the topic matches its `## ` heading.
3. Every case folder has exactly one entry, and every entry without `(planned)` has a folder.
4. A case folder holds only `case.json` (valid), `expect.txt`, `fixture/` and named stdin files. Its `chmod` entries stay inside the sandbox and have octal modes, and a case with `chmod` says `"platforms": ["unix"]`.
5. Goldens use only `$SANDBOX` and `$VERSION` as placeholders.
6. At least one case exists.

When any rule fails, the case trials are not created, and only the failing rules are reported.

## Bless

- `MEMEX_BLESS=1 cargo test --test conformance -- <ID>` rewrites expect.txt for the selected cases that fail or have no golden, and prints `blessed <path>`.
- Bless is refused with `bless refused: CI is set` when `CI` is set.
- Review the result with `git diff tests/cases`. Conformity trials are never blessed.

## Families

- [cli](cli/README.md): `CLI-*`, design §3 and §3.6.
- [show](show/README.md): `SHOW-*`, design §3.3 and §3.6.
- [discovery](discovery/README.md): `DISC-*`, design §4.1.
- [log](log/README.md): `LOG-*`, design §3.4 and §4.3.
- [header](header/README.md): `HDR-*`, design §2.1.
- [intent](intent/README.md): `INT-*`, design §2 and §4.1.
- [collection](collection/README.md): `COLL-*`, design §2.2 and §4.2 (frontmatter collections).
- [caseindex](caseindex/README.md): `CASE-*`, design §2.2.
- [manifest](manifest/README.md): `MAN-*`, design §4.2 (loading manifests, the unreadable-manifest warning, and the kind-gone warning for `kind: markdown`, MAN-LOAD-14).
- [manifest-kinds](manifest-kinds/README.md): `MANK-*`, design §4.2 (the folder and json kinds).
- [ref](ref/README.md): `REF-*`, design §3.1 (ref resolution).
- [cite](cite/README.md): `CITE-*`, design §2.3 (citations and backlinks).
- [search](search/README.md): `SRCH-*`, design §3.2 and §3.6 (words).
- [info](info/README.md): `INFO-*`, design §3.5 and §3.6 (the atlas).
- [audit](audit/README.md): `AUD-*`, design §3.5d and §3.6 (the intent-layer report).
- [tree](tree/README.md): `TREE-*`, design §3.5e and §3.6 (the folder listing).
- [init](init/README.md): `INIT-*`, design §3.5b and §6.1 (setup).
- [port](port/README.md): `PORT-*`, design §4.4 (portability).

## Choices where the rules are silent

- The conformity messages name the rule as `rule1` to `rule6`, and paths use `/` on every OS.
- Rule 1 also requires the hub README.md, and each family's README.md and CASES.md.
- Rule 4 also checks `steps` is not empty, `stdin` and `stdin_file` are not both set, a stdin_file is a plain file name that exists, setup argv starts with `git` or `memex`, every step `cwd` and `snapshot` path stays inside the sandbox (relative, no `..`, no leading separator, no drive or `:`), and `platforms` is not empty. `expect.txt` is optional, and a missing one fails its case.
- Rule 5 looks for `$` followed by `[A-Z_]` and the run of `[A-Za-z0-9_]` after it. A `$ ` prompt is never a token. Bless also refuses a golden that rule 5 would reject.
- Platform `unix` covers every OS that is not Windows.
- Rule 4 also checks each `chmod` path stays inside the sandbox, each mode is octal, and a case with `chmod` is `platforms: ["unix"]`. Its messages are `chmod N path ...`, `chmod N mode ...` and `chmod needs platforms ["unix"]`.
- The runner's `chmod` restore adds rwx to a folder and rw to a file, rather than setting exact modes, and it never follows a symlink.
- `git: commit` passes `--allow-empty`, so an empty fixture still commits. Runner git runs, including `git` setup runs, use the sandbox environment plus the fixed identity and dates, so setup commits have stable hashes too. A case's `env` overrides them.
- Setup runs use the case environment, not a step's.
- `XDG_CONFIG_HOME` is `home/.config`, a folder under `home`.
- Process output renders a signal end as `exit signal`. A stream that is empty renders `(empty)`, and a stream with a NUL byte or non-UTF-8 bytes renders `(binary)`.
- A snapshot file renders under its base name. An empty file renders `(empty)`, a symlink `(symlink)`, and another non-file `(not a file)`.
- `MEMEX_BLESS` counts only when it is `1`. `CI` counts when it is set at all.
- A failing conformity rule clears the name filter, so a filter cannot hide a failure and leave nothing to run.
- `fill` files are made after the git step, so they are never committed: in a `git: commit` case they are untracked. The runner never replaces a file that the fixture or an earlier fill entry holds. Conformity rule 4 does not check `fill` paths, so a bad one fails its case when the case runs.
