# logs

The log store code. `write.rs` puts one entry in the store: memex's frontmatter, the body verbatim, a file name from the title's slug, and never an overwrite. `prune.rs` is the stub that S1.3 turns into the keep_logs rule. The rules are design §3.4 and §4.3; where the store is comes from `root.rs`.

- `write.rs`: the entry, its frontmatter in fixed key order, the title and slug rules, and the `-2`, `-3` suffixes for a taken name.
- `prune.rs`: `run(store, keep_logs, now)`, called after every write. It deletes nothing until S1.3.
