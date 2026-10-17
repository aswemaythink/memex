# log

The log family covers design §3.4 and §4.3: `memex log`, the ref it prints and its rules. It writes memex's frontmatter, the body as given and a file name from the title's slug; its rules are the body limit and title, frontmatter merging, stream names, flag values, and pruning by size (max_logs_mb) and age (keep_logs). Its prefix is `LOG`. Its cases observe `log` through its stdout, its `--json` output and snapshots of the store. LOG-READ-01 pins that a dot-named entry is read by no read verb, and LOG-PRUNE-19 that `search`, `show` and `info` never prune.

The live cases for the stream-name length, the stdout line, the device names, leading blank lines and the slug are named here one by one, so that the selfhost gate sees each one cited: LOG-WRITE-16, LOG-WRITE-17, LOG-STREAM-06, LOG-BODY-09 and LOG-SLUG-01.
