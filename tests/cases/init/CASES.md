# Init cases

## FRESH

- `INIT-FRESH-01` In a fresh git repo, init creates `.memex/` with `.gitignore`, `config.json` and `collections/.gitkeep`, creates AGENTS.md with the marked block, and prints the four `created` lines, a blank line and the atlas, exit 0.
  Scenario: `repo/` is a git repo holding a README and no memex files. design §3.5b items 1, 2 and 4.

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
  Scenario: AGENTS.md holds two CRLF lines and no markers. design §4.4 Text accepts CRLF; design §3.5b is silent on the endings of a block, so CRLF to match the file is a choice pinned here.

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

## PATH

- `INIT-PATH-01` While the install folder is not on PATH, each run prints `add to PATH: <dir>` after its install line, and the second run reports the identical binary `kept`, exit 0.
  Scenario: two runs with `memex_on_path` false on unix. design §3.5b item 3 (the plain PATH hint).
- `INIT-PATH-02` When a memex executable is on PATH, init installs nothing, reports nothing about the binary and prints no PATH hint, exit 0.
  Scenario: `memex_on_path` is true, and `home/.local` stays missing. design §3.5b item 3 (skip when memex is on PATH).

## INFO

- `INIT-INFO-01` After the report, init prints a blank line and then the atlas of the target, with its intent counts, exit 0.
  Scenario: a git repo with a README and a `src/lib.rs` header; the atlas counts one readme and one header. design §3.5b item 4.
- `INIT-INFO-02` Run from a subfolder, init prints the atlas of the toplevel, so the `here` line is absent, exit 0.
  Scenario: the cwd `repo/src/deep` sits below a `src/README.md` that would give a here line if the atlas read the cwd. design §3.5b item 4 and §3.5 `here` line.

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
