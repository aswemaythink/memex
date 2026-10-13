# log

The log family covers design §3.4 and §4.3: what `memex log` writes, which is memex's frontmatter, the body verbatim, a file name from the title's slug, and the ref it prints, plus the stream grammar. Its prefix is `LOG`. The rules that S1.2 and S1.3 own, such as the body limit, frontmatter merging, reserved streams and pruning, are listed as planned cases with no folder yet. Its cases observe `log` through its stdout, its `--json` output and snapshots of the store.
