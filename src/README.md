# src

`src/` is the crate's source: the `memex` library (`lib.rs`, whose module map is in PLAN.md §4) and the thin `main.rs` binary that turns `memex::run`'s result into the process exit code. Modules land slice by slice, in the order PLAN.md §6 gives.

- `cli.rs`: the argv grammar of design §3, parsed into a typed `Command`, or into the usage error the grammar calls for.
- `grammar.rs`: the grammar's tables: the flags each verb takes, what each flag takes after it, and the words each verb takes.
- `scan.rs`: reads one verb's arguments against its row of `grammar.rs`, with each word and flag value checked as design §3 says.
- `help.rs`: the help texts and the grammar block of design §3, with the usage lines that usage errors print.
- `error.rs`: the fatal error of design §3.6, its codes and exit codes, and its text and JSON renderings.
- `clock.rs`: `MEMEX_NOW` or the system clock, in UTC, and the RFC 3339 and log-stamp formats of design §3.6.
- `root.rs`: where `log` writes (design §4.1): the nearest `.memex/`, the git toplevel and main checkout, and the store path, found through git subprocesses.
- `config.rs`: `.memex/config.json` (design §4.1 item 4): the `keep_logs` value and its warning.
- `logs/`: the log store: writing an entry, pruning old ones and reading them as docs (design §3.4, §4.3).
- `walk.rs`: the ignore walker over the root, with the skip list and `/` labels (design §4.1 item 2).
- `corpus.rs`: the read model (design §3.1, §3.6): every walked file and folder, the docs, the label map and the folder index, and the root that `show` reads from.
- `resolve/`: resolves a ref to a file, a folder or a doc (design §3.1), a path in another root included, or reports it not found or ambiguous; see its README.md.
- `render.rs` and `render/`: the sections and misses that `show` prints, as text and as the `--json` document (design §3.3, §3.6).
- `cite.rs` and `cite/`: the citations a doc makes and the backlinks they give (design §2.3), built once when the corpus loads.
- `intent/`: the readers of headers, READMEs and CASES.md, and the one-liner rule they share (design §2).
- `collections/`: manifests, their flat-YAML frontmatter, and the three kinds; the frontmatter is used now, the kinds are stubs until S2.4 and S2.5 (design §4.2).
- `verbs/`: the verbs that run; `log.rs` is `memex log`, `show.rs` is `memex show`, `search.rs` is `memex search`.
- `index/`: the search index of design §3.2 and PLAN §4, built in memory per run: tokens, stems, BM25F, prefix and one-edit matches, and the ranking.
- `render/search.rs`: `search`'s groups as text, with the `(showing N of M)` headings, and as its `--json` document (design §3.2, §3.6).
