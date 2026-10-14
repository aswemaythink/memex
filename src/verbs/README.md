# verbs

The verbs are the commands that do work, one module each. `log.rs` runs `memex log` (design §3.4): it checks the stream name, reads the body, places the entry and returns the ref, or the path as JSON with `--json`. `show.rs` runs `memex show` (design §3.3): it loads the corpus for the cwd's root, resolves each ref, builds its section and reports the sections and misses, with exit 1 when a ref failed. `lib.rs` dispatches to both; the other verbs are not built yet and still print their not-implemented line.
