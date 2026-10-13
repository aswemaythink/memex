# verbs

The verbs are the commands that do work, one module each. `log.rs` runs `memex log` (design §3.4): it checks the stream name, reads the body, places the entry and returns the ref, or the path as JSON with `--json`. `lib.rs` dispatches to it; the other verbs are not built yet and still print their not-implemented line.
