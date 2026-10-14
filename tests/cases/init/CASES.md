# Init cases

## FRESH

- `INIT-FRESH-01` In a fresh git repo, init creates `.memex/` with `.gitignore`, `config.json` and `collections/.gitkeep`, creates AGENTS.md with the marked block, and prints the four `created` lines, a blank line and the atlas, exit 0.
  Scenario: `repo/` is a git repo holding a README and no memex files. design §3.5b items 1, 2 and 4.
- `INIT-FRESH-02` In a fresh git repo, init writes `.memex/config.json` as exactly `{"keep_logs": "3d"}` and one newline, exit 0.
  Scenario: `repo/` is a git repo with no files; the snapshot shows the bytes, and a missing final newline would show as `\ no newline at end`. design §3.5b item 1 (`config.json` holds `{"keep_logs": "3d"}`).

## KEEP

- `INIT-KEEP-01` An existing `.memex/.gitignore` and `config.json` keep their own bytes, and a missing `collections/.gitkeep` is created, exit 0.
  Scenario: `config.json` sets `keep_logs` to `7d` and `.gitignore` holds `logs/` only; the snapshot shows both unchanged. design §3.5b item 1 (never overwrites an existing file).
- `INIT-KEEP-02` An existing `.memex/collections/` keeps its `.gitkeep` and its manifest untouched, and reports the `.gitkeep` as `kept`, exit 0.
  Scenario: `collections/` holds `.gitkeep` and `demo.json`; the snapshot lists both with their bytes. design §3.5b item 1.

## AGENTS

- `INIT-AGENTS-01` When AGENTS.md is missing, init creates it holding the begin marker, the snippet and the end marker, and reports `created AGENTS.md`, exit 0.
  Scenario: `.memex/` is complete, so only AGENTS.md is created; the snapshot shows the block. design §3.5b item 2 and §6.
- `INIT-AGENTS-02` When AGENTS.md has no markers, init appends a blank line and the marked block at its end, after a newline the file lacked, and reports `updated AGENTS.md`, exit 0.
  Scenario: AGENTS.md holds `# Agent notes` and one line with no final newline; the user's lines stay above the block. design §3.5b item 2 (append).
- `INIT-AGENTS-03` When the markers hold a stale block, init replaces only the lines between them and reports `updated AGENTS.md`, exit 0.
  Scenario: AGENTS.md has a heading above the markers and a line below them; both stay and the old block is replaced. design §3.5b item 2 (replace).
- `INIT-AGENTS-04` When the block already is the snippet, init reports `kept AGENTS.md` and leaves every byte of the file, exit 0.
  Scenario: AGENTS.md holds a heading, the current block and a last line. design §3.5b item 2 (identical content is kept).
- `INIT-AGENTS-05` When AGENTS.md uses CRLF line endings, the appended block uses CRLF too, so the file keeps one convention, exit 0.
  Scenario: AGENTS.md holds two CRLF lines and no markers. design §4.4 Text: a user file memex edits in place (AGENTS.md, CLAUDE.md) keeps its line endings, so the appended block is CRLF too.

## CLAUDE

- `INIT-CLAUDE-01` When CLAUDE.md lacks an `@AGENTS.md` line, the first init appends it as the final line, after a newline the file lacked, and a second init keeps it, so the import is added once, exit 0.
  Scenario: CLAUDE.md holds two lines with no final newline; two runs in one folder. design §3.5b item 2.
- `INIT-CLAUDE-02` When CLAUDE.md already has the `@AGENTS.md` line, init reports `kept CLAUDE.md` and leaves it unchanged, exit 0.
  Scenario: CLAUDE.md is the single line `@AGENTS.md`. design §3.5b item 2.
- `INIT-CLAUDE-03` Init never creates CLAUDE.md: with none present, it prints no CLAUDE line and the snapshot shows the file missing, exit 0.
  Scenario: a fresh git repo with no CLAUDE.md. design §3.5b item 2 (CLAUDE.md is never created).

## SUB

- `INIT-SUB-01` Run from a subfolder of a git repo, init writes into the toplevel and creates nothing in the subfolder, exit 0.
  Scenario: the cwd is `repo/src/deep`; the lines name the files root-relative to `repo/`, and `repo/src/deep/.memex` is missing afterwards. design §3.5b (inside git, the toplevel).
- `INIT-SUB-02` Outside git, init targets the cwd, so `.memex/` appears in the cwd and not in its parent, exit 0.
  Scenario: `repo/sub` is the cwd of a folder with no git; `repo/.memex` stays missing. design §3.5b (outside git, the cwd).

## INSTALL

- `INIT-INSTALL-01` With no memex on PATH, unix init copies the running binary to `$HOME/.local/bin/memex`, reports it `created`, and prints the PATH hint, exit 0.
  Scenario: `memex_on_path` is false; the snapshot of `home/.local/bin` shows the binary. design §3.5b item 3 and PLAN §1 fix 16.
- `INIT-INSTALL-02` On Windows, init installs to `%LOCALAPPDATA%\Programs\memex\memex.exe`, read from the variable, and prints the PATH hint, exit 0.
  Scenario: the Windows twin of INSTALL-01, with LOCALAPPDATA under the sandbox home. design §3.5b item 3.
- `INIT-INSTALL-03` An installed binary whose bytes differ from the running one is replaced and reported `updated`, exit 0.
  Scenario: `home/.local/bin/memex` holds older text in the fixture. design §3.5b item 3 (replaced only if the bytes differ).
- `INIT-INSTALL-04` An empty or relative `$HOME` is an environment error: init prints the lines it made, names the HOME variable on stderr and exits 3, and writes no binary under the cwd, unix only.
  Scenario: one run with `HOME` empty, then one with `HOME=home`; `repo/.local` and `repo/home` stay missing. design §3.5b item 3 (the install dir comes from `$HOME`) and §3.5b (exit 3 on an I/O failure).

## PATH

- `INIT-PATH-01` While the install folder is not on PATH, each run prints `add to PATH: <dir>` after its install line, and the second run reports the identical binary `kept`, exit 0.
  Scenario: two runs with `memex_on_path` false on unix. design §3.5b item 3 (the plain PATH hint).
- `INIT-PATH-02` When a memex executable is on PATH, init installs nothing, reports nothing about the binary and prints no PATH hint, exit 0.
  Scenario: `memex_on_path` is true, and `home/.local` stays missing. design §3.5b item 3 (skip when memex is on PATH).
- `INIT-PATH-03` A memex in a `node_modules/.bin` folder on PATH is npx's shim and does not count: init installs the binary, reports it `created`, and prints the PATH hint, exit 0, unix only.
  Scenario: PATH is `node_modules/.bin:/usr/bin:/bin`, and a patch puts an executable `memex` in `repo/node_modules/.bin`. A case cannot name a sandbox path, so the entry is relative to the run's folder, `repo/`. design §3.5b item 3 (PATH entries inside a `node_modules/.bin` folder are ignored). The Windows names are pinned by INIT-PATH-04 and INIT-PATH-05, which run on Windows only.
- `INIT-PATH-04` On Windows, a `memex.cmd` in a folder on PATH counts as a memex, so init installs nothing, prints no PATH hint and exits 0.
  Scenario: PATH is `bin`, a folder of the cwd `repo/` that the fixture gives a `memex.cmd`; the snapshot shows the install folder missing. design §3.5b item 3 (on Windows `memex.exe` or `memex.cmd` counts). The golden is derived by hand and is unverified until Windows CI runs.
- `INIT-PATH-05` On Windows, a `memex.cmd` under `node_modules\.bin` on PATH does not count: init installs the binary, reports it `created` and prints the PATH hint, exit 0.
  Scenario: PATH is `node_modules\.bin`, and the fixture gives `repo/node_modules/.bin/memex.cmd`; the snapshot shows the installed `memex.exe`. design §3.5b item 3 (npx's shim folder is ignored, and on Windows the match is the same). The golden is derived by hand and is unverified until Windows CI runs.

## INFO

- `INIT-INFO-01` After the report, init prints a blank line and then the atlas of the target, with its intent counts, exit 0.
  Scenario: a git repo with a README and a `src/lib.rs` header; the atlas counts one readme and one header. design §3.5b item 4.
- `INIT-INFO-02` Run from a subfolder, init prints the atlas of the toplevel, so the `here` line is absent, exit 0.
  Scenario: the cwd `repo/src/deep` sits below a `src/README.md` that would give a here line if the atlas read the cwd. design §3.5b item 4 and §3.5 `here` line.
- `INIT-INFO-03` The atlas of init ignores `$MEMEX_AGENT`: an invalid id neither fails init nor warns, and a valid id with messages gives no `to` line, exit 0 both times.
  Scenario: two runs, with `MEMEX_AGENT` set to `bad id` and then to `cook-2`, which two logs address. init has no `--agent`, and an id that `info` would refuse must not exit 2 after the files are written; design §3.5b item 4: the atlas is printed without an agent id, so its output depends only on the repo.

## RERUN

- `INIT-RERUN-01` A second init in the same folder reports every file `kept` and writes none of them, exit 0.
  Scenario: two runs in a git repo that has a CLAUDE.md holding `@AGENTS.md`. design §3.5b (idempotent) and PLAN §3 (init on this repo reports everything unchanged).

## IO

- `INIT-IO-01` When AGENTS.md is a folder, init prints the `.memex/` lines it made, then names the file and its io::ErrorKind on stderr, and exits 3 with no atlas, unix only.
  Scenario: `repo/AGENTS.md` is a folder holding one file, so reading it fails with IsADirectory. design §3.5b (exit 3 on an I/O failure) and PLAN §1 fix 17 (memex's message, the kind and the path, never the OS text).

## DAMAGE

- `INIT-DAMAGE-01` A begin marker with no end marker changes nothing: AGENTS.md is reported `kept`, the warning goes to stderr, and exit is 0.
  Scenario: AGENTS.md holds a begin marker and a block with no end. design §3.5b item 2 and PLAN §1 fix 16; the warning text is pinned here.
- `INIT-DAMAGE-02` An end marker before the begin marker changes nothing, and the warning goes to stderr, exit 0.
  Scenario: AGENTS.md has an end marker, a line, then a begin marker. design §3.5b item 2.
- `INIT-DAMAGE-03` Two pairs of markers change nothing, and the warning goes to stderr, exit 0.
  Scenario: AGENTS.md holds two complete blocks. design §3.5b item 2 (more than one pair is damage).

## FENCE

- `INIT-FENCE-01` A marker pair inside a backtick fence is not the block: init appends a real block after the document and leaves the fenced example as it is, exit 0.
  Scenario: AGENTS.md holds a fenced example of the markers and no block; the snapshot shows the example unchanged and the block at the end. design §2.3 (citations are never extracted from fenced code blocks) and §3.5b item 2 (markers count only outside fenced code blocks).
- `INIT-FENCE-02` A CLAUDE.md whose only `@AGENTS.md` line sits in a fence gets the import as its final line and reports `updated CLAUDE.md`, exit 0.
  Scenario: CLAUDE.md holds a fenced `@AGENTS.md` line and no import; AGENTS.md is missing, so init creates it first. design §3.5b item 2 (the import line counts only outside fenced code blocks).
- `INIT-FENCE-03` A closed fence above the block does not hide it: init replaces only the lines between the real markers and leaves the fenced example, exit 0.
  Scenario: AGENTS.md holds a fenced example of the markers, then the real block with stale text. design §2.3 (a fence closes at its closing line) and §3.5b item 2 (replace).
- `INIT-FENCE-04` A tilde fence hides a marker pair the way a backtick fence does, so init appends a real block, exit 0.
  Scenario: AGENTS.md holds the markers inside a `~~~` fence and no block. design §2.3 (a fence is three or more backticks or tildes).
- `INIT-FENCE-05` A code fence that is never closed hides the end of AGENTS.md, so init writes nothing: it reports `kept AGENTS.md`, warns on stderr, and exits 0.
  Scenario: AGENTS.md ends inside a fence that is never closed and holds no block. design §2.3 (an unclosed fence runs to the end of the doc). Design §3.5b is silent on this case: a block appended there would sit inside the fence, so the next run would append another, and the warning and the kept file are a choice pinned here.
- `INIT-FENCE-06` A code fence that is never closed hides the end of CLAUDE.md, so init writes no import: it reports `kept CLAUDE.md`, warns on stderr, and exits 0.
  Scenario: CLAUDE.md ends inside a fence that is never closed and holds `@AGENTS.md`; AGENTS.md is missing and gets created. design §2.3 and §3.5b item 2. The warning is a choice pinned here, for the reason given at INIT-FENCE-05.
- `INIT-FENCE-07` A line indented four spaces is code, not a fence, so a marker pair under it is the real block and is replaced, exit 0.
  Scenario: AGENTS.md holds a four-space indented fence line, then an old block whose text is replaced. design §2.3 (a fence is indented at most three spaces).

## LINK

- `INIT-LINK-01` A symlink named AGENTS.md is never written through: init warns that it is a symlink, reports it `kept`, and leaves the file it names unchanged, exit 0, unix only.
  Scenario: AGENTS.md is a symlink to `outside/shared.md`, a file outside the repo, made by a git patch as LOG-PRUNE-14 does; the snapshot of `repo` shows the link as `(symlink)` and the file it names is unchanged. design §3.5b item 2 (never writes through a symlinked AGENTS.md or CLAUDE.md: it warns and leaves them alone).
- `INIT-LINK-02` A symlink named CLAUDE.md is never written through: init warns that it is a symlink, reports it `kept`, and leaves the file it names unchanged, while AGENTS.md is created as usual, exit 0, unix only.
  Scenario: CLAUDE.md is a symlink to `outside/shared.md`, and AGENTS.md is missing; the snapshot of `repo` lists the link as `(symlink)` next to the new block. design §3.5b item 2.

## NEST

- `INIT-NEST-01` Outside git, init run from `repo/outer/inner` targets `repo/outer`, the nearest folder above it with a `.memex/`: its report opens with the line `using the store at <root>; to start a new one here instead: mkdir .memex && memex init`, with `<root>` the absolute, `/`-separated path of `repo/outer`; its `.memex/` files are reported `kept`, AGENTS.md is created there, and nothing is created in `inner`, exit 0.
  Scenario: `repo/outer/.memex/` is complete and `repo/outer/inner/` holds one file, with no git repo above the sandbox; the snapshot shows `inner` as it was and the store unchanged. design §3.5b (outside git, the root that discovery finds from the cwd, so init never shadows a store above it, and the report names that root on its first line when it is above the cwd) and §4.1 item 1.
- `INIT-NEST-02` Inside git, init run from a subfolder writes into the git toplevel even when a `.memex/` sits above the repo, so the store above is left as it is, exit 0.
  Scenario: the sandbox root holds `.memex/config.json` above the git repo `repo/`, and the cwd is `repo/sub`; `repo/sub/.memex` stays missing, `repo/.memex/` is created, and the store above is unchanged. design §3.5b (inside git, the toplevel).
- `INIT-NEST-03` Outside git, init run from `$HOME/project` adopts the complete `.memex/` in `$HOME`: its report opens with the same line, naming `$HOME` as `<root>`, AGENTS.md is created in `$HOME`, and nothing is created in `project`, exit 0.
  Scenario: `home/.memex/` is complete (the sandbox's `$HOME` is `home/`) and `home/project/` holds one file, with no git repo above the sandbox; the snapshot shows `project` as it was and the store unchanged. design §3.5b (a `.memex/` in `$HOME` is adopted like any other, and named the same way) and §4.1 item 1 (outside git the walk reaches `$HOME`).

## PASTE

- `INIT-PASTE-01` When AGENTS.md has no markers but holds the snippet's heading line outside a fence, init leaves the file byte-identical, reports `kept AGENTS.md`, prints the paste warning on stderr, and exits 0.
  Scenario: AGENTS.md holds a title and then the whole snippet pasted by hand, with no markers; the snapshot shows the file unchanged. design §3.5b item 2 (the warning's exact text). The `kept` report follows the damaged-marker rule, since nothing changed.
- `INIT-PASTE-02` A heading line that sits only inside a fenced code block is not a paste: init appends the marked block at the end as usual, reports `updated AGENTS.md`, and prints no warning, exit 0.
  Scenario: AGENTS.md holds a fenced `md` example that contains the heading line, and no markers; the snapshot shows the example unchanged and the block at the end. design §2.3 (a fence hides its lines) and §3.5b item 2 (the heading counts only outside fenced code).
