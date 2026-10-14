# log

The log family covers design §3.4 and §4.3: what `memex log` writes, which is memex's frontmatter, the body verbatim, a file name from the title's slug, and the ref it prints and the rules on top: the body limit and title, frontmatter merging, stream names, flag values, and pruning by keep_logs. Its prefix is `LOG`. Its cases observe `log` through its stdout, its `--json` output and snapshots of the store.

The live cases for the stream-name length, the stdout line, the device names, leading blank lines and the slug are named here one by one, so that the selfhost gate sees each one cited: LOG-WRITE-16, LOG-WRITE-17, LOG-STREAM-06, LOG-BODY-09 and LOG-SLUG-01.
