# logs

The log store code. Four modules check a `memex log` entry before anything is written: `body.rs` reads stdin and refuses a terminal, an empty body, a body over 16 KiB, or one with no title; `front.rs` splits the agent's own leading `---` block off the body and refuses memex's keys and repeats inside it; `stream.rs` checks the stream name, its grammar, the reserved built-in names and any manifest that already owns the name; `values.rs` checks the values of `--to`, `--tag` and the agent id and drops repeats from the lists. Then `write.rs` puts one entry in the store: memex's frontmatter with the agent's lines merged in, the body verbatim, a file name from the title's slug, and never an overwrite. `prune.rs` is the stub that S1.3 turns into the keep_logs rule. The rules are design §3.4 and §4.3; where the store is comes from `root.rs`.

- `body.rs`: `read_stdin` and `read` (the terminal refusal is injected so a unit test can pin it), `check_title`, and the 16 KiB limit, counted before CRLF becomes LF.
- `front.rs`: `split` returns the agent block's lines and the body after its closing fence. A key is a column-0 identifier followed by a colon; every other line is kept as continuation.
- `stream.rs`: `check_grammar`, `check_reserved` and `check_collections`. The last looks for `.memex/collections/<stream>.json` under the nearest `.memex/` above the cwd and under the store owner's `.memex/`.
- `values.rs`: `list` for the comma lists, `agent` for `--agent` or `$MEMEX_AGENT` (only the one in use is checked). Each value matches `[A-Za-z0-9._@/+-]{1,64}`.
- `write.rs`: the entry, its frontmatter in fixed key order, the title and slug rules, and the `-2`, `-3` suffixes for a taken name.
- `prune.rs`: `run(store, keep_logs, now)`, called after every write. It deletes nothing until S1.3.
