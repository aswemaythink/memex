---
memex: cases
---
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

- `DISC-GIT-01` With git missing from PATH, a log in a repo writes the entry with the repo's branch, read from its HEAD, and no commit: the repo's `.memex/` takes the entry.
  design §4.4 git: without git the branch comes from HEAD and the commit is omitted. Run: memex log impl with PATH=/nonexistent (unix only); snapshot of repo/.memex/logs.
- `DISC-GIT-02` With git missing from PATH, a log in a repo whose HEAD is detached writes an entry with neither a branch nor a commit.
  design §4.4 git: a HEAD that holds a commit name names no branch. Run: after `git checkout --detach`, memex log impl with PATH=/nonexistent (unix only); snapshot of repo/.memex/logs.
- `DISC-GIT-03` With git missing from PATH, a HEAD that names `refs/heads/.invalid` (reftable's placeholder) gives the entry no branch.
  design §4.4 git: a branch name with a part that starts with a dot is no branch. Run: memex log impl with PATH=/nonexistent (unix only), in a repo whose `.git` folder holds only a HEAD with `ref: refs/heads/.invalid`; snapshot of repo/.memex/logs.

## CONFIG

- `DISC-CONFIG-01` A `.memex/config.json` that is not valid JSON prints one warning on stderr naming the file, and the entry is still written with the defaults.
  design §4.1 item 4; PLAN §1 fix 18. Run: memex log impl with an invalid config.json.
- `DISC-CONFIG-02` A `.memex/config.json` with an unknown key prints one warning on stderr naming the key, and the entry is still written with the defaults.
  design §4.1 item 4; PLAN §1 fix 18. Run: memex log impl with an extra key beside keep_logs.
- `DISC-CONFIG-03` A `.memex/config.json` that is a JSON string prints one warning on stderr naming the file, and the entry is still written.
  design §4.1 item 4 (an object of known keys) and §4.3 Cleanup (never delete on config it can't read). Run: memex log impl with config.json `"3d"`.
- `DISC-CONFIG-04` A `.memex/config.json` that is a JSON number prints one warning on stderr naming the file, and the entry is still written.
  design §4.1 item 4 (an object of known keys) and §4.3 Cleanup (never delete on config it can't read). Run: memex log impl with config.json `5`.

## NOGIT

- `DISC-NOGIT-01` With git missing from PATH, `log` in a linked worktree of an ordinary checkout writes to the main checkout's `.memex/logs/`, and nothing in the worktree: the entry has the worktree's branch, its toplevel as `worktree`, and no commit. Text, then --json.
  design §4.4 git: the layout names the main checkout (the worktree's `commondir` names a `.git` folder), so the store is its counterpart there, as with git. Run: memex log impl, text and then --json, from wt with PATH=/nonexistent, unix only; snapshots of repo/.memex and wt/.memex.
- `DISC-NOGIT-02` With git missing from PATH, `info` in the same linked worktree still works: exit 0, the overview on stdout and nothing on stderr, in text and with `--json`.
  design §4.4 git: read verbs are unchanged. Run: memex info, text and then --json, from wt with PATH=/nonexistent, unix only.
- `DISC-NOGIT-03` With git missing from PATH, `log` from a folder inside a linked worktree that holds its own `.memex/` writes to the same folder of the main checkout, as DISC-WT-02 does with git, and nothing in the worktree's folder.
  design §4.1 item 3 and §4.4 git: the nearest `.git` entry, found by listing, is the worktree's file and names the main checkout, so the folder maps to its counterpart there. Run: memex log impl from wt/sub, which has a `.memex/`, with PATH=/nonexistent, unix only; snapshots of repo/sub/.memex and wt/sub/.memex.
- `DISC-NOGIT-04` With git missing from PATH, `log` in a linked worktree of a bare repo is refused: exit 3 with `memex: cannot find the main checkout's log store: git did not answer and this linked worktree's repository is not a .git folder; run memex log where git works`, and with `--json` the envelope has code `io`. Neither store gains an entry or a `.gitignore`.
  design §4.4 git: the common dir is not named `.git`, so the layout cannot place the store, and the worktree's own store would be lost with it. Run: memex log impl, text and then --json, from wt-bare with PATH=/nonexistent, unix only; snapshots of bare.git/memex and wt-bare/.memex.
- `DISC-NOGIT-05` With git present but failing (GIT_DIR=/nonexistent makes every rev-parse exit 128), `log` in a linked worktree writes as DISC-NOGIT-01 does.
  design §4.4 git: a failing git gives no answer, as a missing one does. Run: the DISC-NOGIT-01 steps with env GIT_DIR=/nonexistent, and the same snapshots.
- `DISC-NOGIT-06` With git missing from PATH, `log` in a submodule writes to the submodule's own `.memex/` and is not refused, since its `.git` file names a git dir with no `commondir`.
  design §4.4 git: a `.git` file whose git dir has no commondir is a main checkout, so the store is the root's own. Run: memex log impl from repo/sub, a submodule, with PATH=/nonexistent, unix only; snapshot of repo/sub/.memex.
- `DISC-NOGIT-07` With git missing from PATH, `log` in a folder whose `.git` is a file naming a missing git dir is refused with the DISC-NOGIT-04 message, and writes nothing.
  design §4.4 git: the commondir cannot be read, so the layout cannot place the store. Run: memex log impl from repo/sub, where repo/.git is a file whose gitdir line names a folder that does not exist, with PATH=/nonexistent, unix only; snapshot of repo/.memex.

## TOP

- `DISC-TOP-01` With git missing from PATH, `info` from a subfolder of a repo with no `.memex/` names the repo's toplevel as the root, not the subfolder, in text and with `--json`, exit 0.
  design §4.1 item 1: the toplevel is found by listing, so the root does not need git. Run: memex info, text and then --json, from repo/sub with PATH=/nonexistent, unix only.
- `DISC-TOP-02` With git missing from PATH and a `.memex/` above the repo, `info` from repo/sub names the repo as the root, and `log` exits 3 with the no_root message and writes nothing, in text and with `--json`.
  design §4.1 item 1: the walk stops at the toplevel, so a `.memex/` above a repo never captures it. Run: memex info and memex log impl, text and then --json, from repo/sub with PATH=/nonexistent, unix only; snapshots of repo/.memex and .memex/logs.
- `DISC-TOP-03` With git present but failing, `info` from repo/sub names the repo as the root, and `log` exits 3 with the no_root message and writes nothing, as in DISC-TOP-02.
  design §4.4 git: a failing git does not change the root, since the toplevel is found by listing. Run: the DISC-TOP-02 steps with env GIT_DIR=/nonexistent, which makes the real git exit 128 on every rev-parse.
- `DISC-TOP-04` A `.git` folder that holds no repository, in a subfolder of a repo, bounds the root there: `info` from inside that subfolder names the subfolder as the root, not the repo whose `.memex/` lies above it, in text and with `--json`, exit 0.
  design §4.1 item 1: a `.git` entry git would reject still marks a toplevel. Run: memex info, text and then --json, from repo/sub, where repo/sub/.git is a folder holding one placeholder file (git cannot store an empty folder, and skips this one as it would an empty one) and repo/.memex/ exists.
- `DISC-TOP-05` Inside the `.git` folder of a repo with no `.memex/`, `info` names the repo as the root, and `log` exits 3 with the no_root message and writes nothing.
  design §4.1 item 1: a cwd inside `.git` belongs to the repo that holds it. Run: memex info and memex log impl, text and then --json, from repo/.git of a committed repo; snapshot of repo/.memex.
- `DISC-TOP-06` A `.git` file that names a missing git dir still marks its folder as the toplevel: `info` from its subfolder names that folder as the root, in text and with `--json`, exit 0.
  design §4.1 item 1: a `.git` entry git would reject still marks a toplevel. Run: memex info, text and then --json, from repo/sub, where repo/.git is a file whose gitdir line names a folder that does not exist.
- `DISC-TOP-07` With git present, `log` inside the DISC-TOP-04 subfolder exits 3 with the no_root message and writes nothing to the repo's `.memex/` above it, in text and with `--json`.
  design §4.1 items 1 and 3: git answers for the outer repo's main checkout, whose store is the root's own, so a `.memex/` above the bound does not capture it. Run: memex log impl, text and then --json, from repo/sub of the DISC-TOP-04 layout; snapshot of repo/.memex.
- `DISC-TOP-08` With git present, `log` from a subfolder of a linked worktree whose `.git` is a folder that holds no repository exits 3 with the no_root message, in text and with `--json`, and the main checkout's `.memex/` gains nothing.
  design §4.1 items 1 and 3: the store maps to a main checkout only when the toplevel's own `.git` is a file, so the bounded subfolder keeps its own store, as DISC-TOP-07 does in a main checkout. Run: memex log impl, text and then --json, from wt/sub, where wt is a linked worktree of repo (repo/.memex/ exists) and wt/sub/.git is a folder holding only a nested `.git` (made by `git init ../wt/sub/.git`, which git skips as no repository); snapshots of repo/.memex and wt/sub/.memex.

## IGNORE

- `DISC-IGNORE-01` With git missing from PATH, a README that `.gitignore` names is absent from `search --in readmes`, and a README it does not name is found.
  design §4.1 item 2: the ignore walker honours `.gitignore` and `.ignore`, and §4.4 git: nothing in the walk needs git. Run: memex search secretword --in readmes, then search publicword --in readmes, with PATH=/nonexistent, unix only; the first README is git-ignored.

## BAREDOT

- `DISC-BAREDOT-01` A bare repo named `.git` does not move the store: `info`, then `log` in text and then --json, from its root, use the root's own `.memex/logs/`, and nothing is written under the bare repo's `memex/`; the entry takes HEAD's branch and no commit, as HEAD is unborn.
  design §4.4 git: a `.git` folder means the root's own store, git is not asked for it, and a bare repo whose folder is named `.git` is an accepted limit. Run: memex info, then memex log impl, text and then --json, from repo, where repo/.git is a bare repo made by `git init --bare` on an unborn main; snapshots of repo/.memex and repo/.git/memex.
- `DISC-BAREDOT-02` With `core.bare` set to true in a committed checkout, `info`, then `log` in text and then --json, from its root, use the root's own `.memex/logs/`, and nothing is written under `.git/memex/`; the entry still takes HEAD's branch and commit.
  design §4.4 git: a `.git` folder means the root's own store, git is not asked for it, and `core.bare=true` is an accepted limit. Run: memex info, then memex log impl, text and then --json, from repo after `git config core.bare true`; snapshots of repo/.memex and repo/.git/memex.
- `DISC-BAREDOT-03` With `GIT_DIR` set to a bare clone of a committed checkout, `info`, then `log` in text and then --json, from the checkout's root, use the root's own `.memex/logs/`, and nothing is written under the clone's `memex/`; the entry takes the clone's branch and commit, which are HEAD's.
  design §4.1 item 1 and §4.4 git: git's environment is not consulted, `GIT_DIR` is an accepted limit, and a `.git` folder means the root's own store without a git call. Run: memex info, then memex log impl, text and then --json, from repo with GIT_DIR=../bare.git, a bare clone of repo; snapshots of repo/.memex and bare.git/memex.
- `DISC-BAREDOT-04` With `GIT_DIR` set to a bare repo, and a `.memex/` in a folder with no `.git` entry above it, `info`, then `log` in text and then --json, from that folder, use the bare repo's `memex/logs/`, which is git's answer, and the folder's `.memex/` gets no store.
  design §4.4 git: with no toplevel, git is asked for the store when a `.memex/` is in reach, so a bare repo keeps its store in its git dir (§4.1 item 3). Run: memex info, then memex log impl, text and then --json, from repo/plain with GIT_DIR=../bare.git, an empty bare repo made in repo; snapshots of repo/plain/.memex and repo/bare.git/memex.
- `DISC-BAREDOT-05` A cwd inside a bare repo, with no `.git` entry above it and a `.memex/` in the folder above, makes `info`, then `log` in text and then --json, use the bare repo's `memex/logs/`, and writes no `.gitignore` beside the root's `.memex/`.
  design §4.1 item 3 and §4.4 git: with no toplevel and a `.memex/` in reach, git's answer places a bare repo's store in its git dir, which gets no `.gitignore`. Run: memex info, then memex log impl, text and then --json, from repo/bare.git, a bare repo made by `git init --bare` in repo, where repo/.memex holds config.json; snapshots of repo/.memex and repo/bare.git/memex.
