# Install memex

You are an agent installing memex in the repo you are working in, for the user who pasted this. memex is a CLI that reads a repo's intent layer: a README per folder, a header comment atop each file, Markdown collections, and logs agents leave each other. Docs: https://aswemaythink.github.io/memex/

Work through the steps in order. Run commands yourself; ask the user only where a step says so. Change nothing the user has not chosen.

## 1. Install and set up

1. Check for the binary: `memex --version`. If it prints a version, skip to `memex init` below.
2. Otherwise pick one route, the first that fits this machine:
   - Node is installed: `npx @aswemaythink/memex init`. This installs the binary and runs init in one step.
   - Homebrew is installed: `brew install aswemaythink/tap/memex`
   - Rust is installed: `cargo install agent-memex`
   - macOS or Linux: `curl -LsSf https://github.com/aswemaythink/memex/releases/latest/download/agent-memex-installer.sh | sh`
   - Windows: `powershell -c "irm https://github.com/aswemaythink/memex/releases/latest/download/agent-memex-installer.ps1 | iex"`
3. Run `memex init` at the repo root (not needed after the npx route). It is safe to repeat. It creates `.memex/`, writes a marked block into `AGENTS.md` telling agents to read intent before code, adds `@AGENTS.md` to an existing `CLAUDE.md`, and prints `memex info`.
4. If init prints `add to PATH: <dir>`, tell the user, and use that full path for the rest of this session.

## 2. Look at the repo

Before asking anything, get the facts:

- `memex info`: what the graph and collections already hold.
- `memex tree`: the folders and files two levels down, with their one-liners and how many files lack a header.
- `memex audit`: folders with no README, files with no header, broken references.
- The top-level folders, whether they have READMEs, and whether there is a test suite and where.

## 3. Ask the user

Ask these in one message. Give each a recommended option, and skip any that does not apply.

1. **Claude Code plugin** (only if you are Claude Code). It adds a memex skill and a start hook that prints `memex info` each session. (a) Yes: the user types `/plugin marketplace add aswemaythink/memex`, then `/plugin install memex@memex`. (b) No.
2. **Seed the intent layer.** Name the counts `memex audit` found. (a) READMEs for the top-level folders that lack one, and headers for the files in them. (b) READMEs for the top-level folders only. (c) Leave it; `memex audit` stays the to-do list.
3. **Tests as a collection** (only if there is a test suite). (a) Start a `CASES.md` for one area the user names: each test gets an id such as `AUTH-001` in its name or a comment, and an entry stating its claim. (b) Not now.
4. **Orchestration** (only if the user runs an orchestrator with worker agents). (a) Add to the worker instructions: end each run with `memex log <stream> --agent <your-id>` holding the full report, and hand back one line, `report: <ref>`. (b) No.

## 4. Do what was chosen

Write intent from the code you read, never from guesses:

- **README.md:** line 1 is `# Name`; then a paragraph whose first sentence says what the folder is.
- **Header:** the first comment block in the file; its first sentence states the file's purpose.
- **CASES.md:** starts with frontmatter `memex: cases`; one list item per test, opening with its id in backticks, then the claim, for example `` - `AUTH-001` Expired tokens are refused. ``
- **Cite, don't copy:** name ids and paths instead of repeating what they say.

Keep the first sentence of every README and header to 100 characters or fewer.

## 5. Check and report

1. Run `memex audit` again and fix what it lists for the files you touched.
2. Run `memex info`.
3. Tell the user, briefly: the install route used, what init wrote, each file you created or changed, and what `memex audit` still lists. Leave the changes uncommitted unless the user asks.

To uninstall later: delete `.memex/` and the block between `<!-- memex:begin -->` and `<!-- memex:end -->` in `AGENTS.md`.
