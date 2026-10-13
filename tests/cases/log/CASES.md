# Log cases

## WRITE

- `LOG-WRITE-01` A basic entry gets the frontmatter `stream`, `time`, `branch` and `commit`, a file named from the title's slug, and its ref on stdout.
  design §3.4 Output and §4.3; PLAN §1 fixes 7 and 8. Run: memex log impl in a repo on main, with a `# Heading` title.
- `LOG-WRITE-02` `--agent` sets the `agent` field.
  design §3.4 Agent id. Run: memex log impl --agent cook-2.
- `LOG-WRITE-03` With no `--agent`, `$MEMEX_AGENT` sets the `agent` field.
  design §3.4 Agent id. Run: memex log impl with MEMEX_AGENT=scout-1.
- `LOG-WRITE-04` `--agent` takes precedence over `$MEMEX_AGENT`.
  design §3.4 Agent id: `--agent`, then `$MEMEX_AGENT`. Run: memex log impl --agent cook-2 with MEMEX_AGENT=scout-1.
- `LOG-WRITE-05` `--to` and `--tag` write `to` and `tags` as lists, in the order given.
  design §3.4 Extra fields and the §4.3 example. Run: memex log impl --to orchestrator,cook-3 --tag show,namer.
- `LOG-WRITE-06` In a linked worktree the frontmatter has `worktree`, the toplevel of that worktree, and the worktree's `branch`, and the entry goes to the main checkout's store.
  design §4.3 frontmatter and §3.4 Output. Run: memex log impl from the linked worktree.
- `LOG-WRITE-07` With `--json`, stdout is one line `{"ref":...,"path":...}`, and the path is absolute with `/` separators.
  design §3.6 JSON, log. Run: memex log impl --json.
- `LOG-WRITE-08` A second entry with the same title at the same MEMEX_NOW gets the `-2` suffix, and the first file is not overwritten.
  design §4.3 concurrency: a noclobber persist, retried as `-2`, `-3`. Run: memex log impl twice with the same body.
- `LOG-WRITE-09` A body with CRLF line endings is written with LF line endings only.
  design §4.4 Text: write `\n`. Run: memex log impl with a CRLF body.
- `LOG-WRITE-10` Outside git, the entry has no `branch` and no `commit`, even when logged from a subfolder.
  design §3.4 Output and §4.3: each field only when known. Run: memex log impl from a subfolder of a plain folder.
- `LOG-WRITE-11` An invalid MEMEX_NOW exits 3 with `invalid MEMEX_NOW:` and the value on stderr, and writes nothing.
  design §3.6 Determinism; PLAN §1 fix 8. Run: memex log impl with MEMEX_NOW set to a non-UTC form, then to a non-date.
- `LOG-WRITE-12` An empty body exits 2 with `log body is empty` on stderr, and writes nothing.
  design §3.4 Body. Run: memex log impl with no stdin.
- `LOG-WRITE-13` A stream name outside `[a-z0-9][a-z0-9-]{0,39}`, such as `Impl` or `../x`, exits 2 and writes nothing.
  design §3.4 Stream name. Run: memex log Impl, then memex log ../x.

## BODY

- `LOG-BODY-01` (planned) A body over 16 KiB exits 2 with "logs are findings: cite artifacts by path" and writes nothing.
  design §3.4 Body. Run: memex log impl with 16 KiB plus one byte on stdin.
- `LOG-BODY-02` (planned) A body whose title is empty, such as one line of only `#` marks, exits 2 and writes nothing.
  design §3.4 Body; PLAN §1 fix 7. Run: memex log impl with stdin `#`.

## FRONT

- `LOG-FRONT-01` (planned) An agent frontmatter block at the top of the body merges with memex's keys into one block, and the body follows it.
  design §3.4 Extra fields and §8 item 5. Run: memex log impl with a body that opens with a `---` block.
- `LOG-FRONT-02` (planned) An agent key that memex owns, such as `time`, exits 2 and writes nothing.
  design §3.4 Extra fields. Run: memex log impl with `time:` in the body's frontmatter.

## STREAM

- `LOG-STREAM-01` (planned) A reserved stream name, `readmes`, `headers`, `cases` or `logs`, exits 2 and writes nothing.
  design §3.4 Stream name and §3.1 built-in collections. Run: memex log logs.
- `LOG-STREAM-02` (planned) A stream name that is an existing manifest's name exits 2 and writes nothing.
  design §3.4 Stream name. Run: memex log with a manifest in .memex/collections.

## PRUNE

- `LOG-PRUNE-01` (planned) A write deletes entries older than keep_logs by their filename timestamp, only `YYYYMMDD-HHMMSS-*.md` files, and removes the stream folders it empties.
  design §4.3 Cleanup; PLAN §1 fix 6. Run: memex log over entries dated before and after the cutoff.
- `LOG-PRUNE-02` (planned) keep_logs `forever` disables pruning.
  design §4.3. Run: memex log with keep_logs set to forever and an old entry present.
- `LOG-PRUNE-03` (planned) An invalid keep_logs prints one warning on stderr and deletes nothing.
  design §4.3; PLAN §1 fix 6. Run: memex log with keep_logs set to soon and an old entry present.
