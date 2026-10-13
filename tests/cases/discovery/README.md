# discovery

The discovery family covers design §4.1: how `log` finds its root and its log store. Its prefix is `DISC`. The root is the nearest folder with a `.memex/`, else the git toplevel, else the cwd. The store is the main checkout's `.memex/logs/` for a linked worktree, and `<git-common-dir>/memex/logs/` for a bare repo. The family also pins what creating the store writes, the fallback when git is missing, and the config warnings of §4.1 item 4. Its cases observe `log` through its `--json` path, its stderr and snapshots of the store.
