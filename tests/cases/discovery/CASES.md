# Discovery cases

## ROOT

- `DISC-ROOT-01` The nearest `.memex/` wins: a log from inside a nested `.memex/` goes to that store, and the outer store is not created.
  design §4.1 item 1. Run: memex log impl --json from repo/sub/deeper, with .memex/ in repo and in repo/sub.
- `DISC-ROOT-02` In a git repo with no `.memex/` anywhere, `log` from a subfolder exits 3 with the no_root message and writes nothing.
  design §4.1 items 1 and 3: the git toplevel is the root, and `log` needs a `.memex/` there or in the main checkout. Run: memex log impl from repo/sub/deeper.
- `DISC-ROOT-03` Outside git with no `.memex/` anywhere, `log` exits 3 with the no_root message and writes nothing.
  design §4.1 item 1: outside git the root is the cwd. Run: memex log impl from a folder outside any repo.
- `DISC-ROOT-04` With `--json`, the no_root error is the JSON envelope with code `no_root` on stdout, exit 3.
  design §3.6 fatal errors and §4.1 item 3. Run: memex log impl --json in a git repo with no `.memex/`.
- `DISC-ROOT-05` Inside a git worktree the search for `.memex/` stops at its toplevel, so a `.memex/` above the repo does not capture it: `info` names the repo as the root, and `log` exits 3 with the no_root message and writes nothing.
  design §4.1 item 1: inside a git worktree the walk stops at the toplevel. Run: memex info and memex log impl, text and --json, from repo, with a .memex/ in the folder above repo.

## WT

- `DISC-WT-01` From a linked worktree whose `.memex/` exists only in the main checkout, the entry goes to the main checkout's `.memex/logs/`, and nothing is written in the worktree.
  design §4.1 item 3: every worktree shares the main checkout's store. Run: memex log impl --json from the linked worktree.
- `DISC-WT-02` A root nested in a folder of a linked worktree maps to the same folder of the main checkout, so the store is that folder's `.memex/logs/`.
  design §4.1 item 3: the same root path in the main worktree. Run: memex log impl --json from wt/sub.
- `DISC-WT-03` `keep_logs` comes from the main checkout's `config.json`, so a log written from a linked worktree prunes the main store's entries older than that setting.
  design §4.1 item 4: the config is the store owner's, the main checkout's for a worktree. Run: memex log impl --json from the linked worktree, with 1d over a 2-day-old and a 1-hour-old entry, then the store snapshot.

## BARE

- `DISC-BARE-01` In a linked worktree of a bare repo, the store is `<git-common-dir>/memex/logs/` inside the bare repo, and no `.gitignore` is written beside the root's `.memex/`.
  design §4.1 item 3: a bare repo has no main checkout, and the store sits in the git dir, so no .gitignore is written. Run: memex log impl --json from the worktree of the bare clone.

## STORE

- `DISC-STORE-01` Creating the store writes `.memex/.gitignore` with the lines `logs/` and `cache/` when that file is missing.
  design §4.1 item 3 and PLAN §1 fix 5. Run: memex log impl in a folder whose .memex/ holds only config.json.
- `DISC-STORE-02` Creating the store leaves an existing `.memex/.gitignore` untouched.
  design §4.1 item 3: the write never overwrites. Run: memex log impl where .memex/.gitignore holds a custom line.

## GIT

- `DISC-GIT-01` With git missing from PATH, a log in a repo is treated as non-git: the repo's `.memex/` takes the entry, and the entry has no branch or commit.
  design §4.4 git: if git is missing, the repo is treated as non-git. Run: memex log impl with PATH=/nonexistent (unix only).

## CONFIG

- `DISC-CONFIG-01` A `.memex/config.json` that is not valid JSON prints one warning on stderr naming the file, and the entry is still written with the defaults.
  design §4.1 item 4; PLAN §1 fix 18. Run: memex log impl with an invalid config.json.
- `DISC-CONFIG-02` A `.memex/config.json` with an unknown key prints one warning on stderr naming the key, and the entry is still written with the defaults.
  design §4.1 item 4; PLAN §1 fix 18. Run: memex log impl with an extra key beside keep_logs.
- `DISC-CONFIG-03` A `.memex/config.json` that is a JSON string prints one warning on stderr naming the file, and the entry is still written.
  design §4.1 item 4 (one key, in an object) and §4.3 (never delete on config it can't read). Run: memex log impl with config.json `"3d"`.
- `DISC-CONFIG-04` A `.memex/config.json` that is a JSON number prints one warning on stderr naming the file, and the entry is still written.
  design §4.1 item 4 (one key, in an object) and §4.3 (never delete on config it can't read). Run: memex log impl with config.json `5`.

## NOGIT

- `DISC-NOGIT-01` With git missing from PATH, `log` in a linked worktree exits 3 with `memex: cannot find the main checkout's log store: git is not on PATH (this folder is a linked worktree); put git on PATH`, and with `--json` the envelope has code `io` on stdout. Neither store gains an entry or a `.gitignore`.
  design §4.4 git: without git, a linked worktree's log is refused, since the entry would be lost with the worktree. Run: memex log impl, text and then --json, from wt with PATH=/nonexistent, unix only; snapshots of repo/.memex and wt/.memex.
- `DISC-NOGIT-02` With git missing from PATH, `info` in the same linked worktree still works: exit 0, the overview on stdout and nothing on stderr, in text and with `--json`.
  design §4.4 git: read verbs are unchanged. Run: memex info, text and then --json, from wt with PATH=/nonexistent, unix only.
- `DISC-NOGIT-03` With git missing from PATH, `log` from a folder inside a linked worktree that holds its own `.memex/` is refused the same way, and writes nothing there.
  design §4.4 git: the nearest `.git` entry at or above the cwd, found by listing, is the worktree's file, so the folder's own `.memex/` does not change the answer. Run: memex log impl from wt/sub, which has a `.memex/`, with PATH=/nonexistent, unix only; snapshot of wt/sub/.memex.
