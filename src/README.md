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
- `logs/`: the log store: writing an entry and pruning old ones (design §3.4, §4.3).
- `verbs/`: the verbs that run; `log.rs` is `memex log`.
