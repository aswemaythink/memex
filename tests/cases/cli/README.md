# cli

The CLI family covers design §3 and §3.6: grammar, help, errors and exit codes. Its prefix is `CLI`. CASES.md declares the case ids, and each case is a folder named by its id. The grammar cases pin help after each verb, `--`, flags, positionals, bare `memex`, unknown verbs and the JSON error envelope. The exit codes are pinned by CLI-EXIT-01 (2), CLI-EXIT-02 and CLI-EXIT-03 (1), CLI-EXIT-04 (3, `no_root`) and CLI-EXIT-05 (3, `io`).

The live case CLI-FLAG-14 pins that `--up 1` before a ref that is not found exits 1 as a not-found error, and is not read as a usage error.
