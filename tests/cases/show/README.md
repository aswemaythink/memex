# show

The show family covers design §3.3 and §3.6: what `memex show` prints for a file, a Markdown file, a folder, `.` and a log ref; its `--up` and `--limit` options; several refs; the partial failures that exit 1; and its `--json` document. Its prefix is `SHOW`. Each case is a folder named by its id, and each carries the same small intent layer as its fixture: a root README, a `src/` folder with a README, headers, a folder without a README, and one log entry. Cases that test a missing `.memex/` leave that folder out.
