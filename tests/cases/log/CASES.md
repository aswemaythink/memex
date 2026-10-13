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

## FLAG

- `LOG-FLAG-01` An item of `--to` that does not match `[A-Za-z0-9._@/+-]{1,64}` exits 2 with `invalid --to value 'cook 2'` and writes nothing.
  design §3.4 Extra fields. Design.md is silent on the value grammar, so this is S1.2's ruling. Run: memex log impl --to orchestrator,cook 2.
- `LOG-FLAG-02` An item of `--tag` that does not match that grammar exits 2 with `invalid --tag value 'a#b'` and writes nothing.
  design §3.4 Extra fields; the grammar is S1.2's ruling. Run: memex log impl --tag show,a#b.
- `LOG-FLAG-03` An `--agent` value that does not match that grammar exits 2 with `invalid --agent value 'cook:2'` and writes nothing.
  design §3.4 Agent id; the grammar is S1.2's ruling. Run: memex log impl --agent cook:2.
- `LOG-FLAG-04` An invalid `$MEMEX_AGENT` exits 2 with `invalid MEMEX_AGENT value 'scout:1'`, but is not checked when `--agent` is given, because the variable is then unused.
  design §3.4 Agent id: `--agent`, then `$MEMEX_AGENT`. Run: memex log impl with MEMEX_AGENT=scout:1, then the same with --agent cook-2.
- `LOG-FLAG-05` A `--to` item of 64 characters is accepted, and one of 65 characters exits 2 with `invalid --to value`.
  design §3.4 Extra fields; the 64-character limit is S1.2's ruling. Run: memex log impl --to with a 64-character id, then with a 65-character id.
- `LOG-FLAG-06` A repeated item in `--to` or `--tag` is written once, in the order it first appears.
  design §3.4 Extra fields and the §4.3 example. Run: memex log impl --to Orch.1@wt/a+b-c,cook-2,Orch.1@wt/a+b-c --tag show,namer,show.

## BODY

- `LOG-BODY-01` A body of 16385 bytes, one over 16 KiB, exits 2 with `log body is over 16 KiB: logs are findings: cite artifacts by path` and writes nothing.
  design §3.4 Body and §8 item 5. Run: memex log impl with a stdin_file of 16385 bytes.
- `LOG-BODY-02` A body whose title is empty, such as one line of only `#` marks, exits 2 with `log body has no title line` and writes nothing.
  design §3.4 Body; PLAN §1 fix 7. Run: memex log impl with stdin `#`.
- `LOG-BODY-03` A body of exactly 16384 bytes is accepted, and its ref is printed with exit 0.
  design §3.4 Body: the maximum is 16 KiB. Run: memex log impl with a stdin_file of 16384 bytes; the golden holds the ref only, since the file is too big to dump.
- `LOG-BODY-04` A body of only spaces, tabs and newlines exits 2 with `log body is empty` and writes nothing.
  design §3.4 Body: required and non-empty, blank counts as empty. Run: memex log impl with stdin of blank lines.
- `LOG-BODY-05` A body whose only text after its frontmatter is blank lines exits 2 with `log body has no title line` and writes nothing.
  design §3.4 Body: the title is the first non-blank line after any frontmatter. Run: memex log impl with a `---` block, then blank lines.

## FRONT

- `LOG-FRONT-01` An agent frontmatter block at the top of the body merges with memex's keys into one block, its lines kept in order with CRLF as LF, and the body follows its closing `---`.
  design §3.4 Extra fields and §8 item 5; §4.2 flat YAML. Run: memex log impl with a CRLF body whose block holds a key, an indented line, a `- item` line and a blank line.
- `LOG-FRONT-02` An agent key that memex owns, such as `time`, exits 2 with `frontmatter key 'time' is set by memex` and writes nothing.
  design §3.4 Extra fields. Run: memex log impl with `time:` in the body's frontmatter.
- `LOG-FRONT-03` A body that opens with a `---` line and never closes the block exits 2 with `frontmatter block is not closed` and writes nothing.
  design §3.4 Extra fields. Run: memex log impl with a body that opens a block and has no later `---` line.
- `LOG-FRONT-04` A key that appears twice in the agent block exits 2 with `frontmatter key 'status' is repeated` and writes nothing.
  design §3.4 Extra fields; §4.2 flat YAML, where a key is one line. Run: memex log impl with `status:` on two lines of the block.
- `LOG-FRONT-05` A `---` line that is not the body's first line is body text, so no frontmatter is read from it.
  design §3.4 Extra fields: the block is leading only. Run: memex log impl with a `# Title` line, then a `---` line and a `time:` line.
- `LOG-FRONT-06` An indented line that looks like a memex key is a continuation line, so it neither collides nor is refused.
  design §4.2 flat YAML: keys are column-0 lines. Run: memex log impl with `  time: 1` indented under an agent key.

## STREAM

- `LOG-STREAM-01` A reserved stream name, `readmes`, `headers`, `cases` or `logs`, exits 2 with `stream name '<name>' is reserved` and writes nothing.
  design §3.4 Stream name and §3.1 built-in collections. Run: memex log logs, then readmes, headers and cases.
- `LOG-STREAM-02` A stream name whose `.memex/collections/<name>.json` manifest exists exits 2 with `stream name '<name>' is a collection` and writes nothing.
  design §3.4 Stream name; §4.2 manifests. Run: memex log rs-notes with that manifest present, then memex log rs, which is accepted because only the exact file name counts.
- `LOG-STREAM-03` From a linked worktree, a manifest that exists only in the main checkout's `.memex/collections` also makes its name a collection.
  design §4.1 item 3 (the store owner is the main checkout) and §3.4 Stream name. Run: memex log rs-notes from a linked worktree, where `.memex/.gitignore` keeps the manifest out of git.

## PRUNE

- `LOG-PRUNE-01` (planned) A write deletes entries older than keep_logs by their filename timestamp, only `YYYYMMDD-HHMMSS-*.md` files, and removes the stream folders it empties.
  design §4.3 Cleanup; PLAN §1 fix 6. Run: memex log over entries dated before and after the cutoff.
- `LOG-PRUNE-02` (planned) keep_logs `forever` disables pruning.
  design §4.3. Run: memex log with keep_logs set to forever and an old entry present.
- `LOG-PRUNE-03` (planned) An invalid keep_logs prints one warning on stderr and deletes nothing.
  design §4.3; PLAN §1 fix 6. Run: memex log with keep_logs set to soon and an old entry present.
