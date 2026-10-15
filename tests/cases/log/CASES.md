---
memex: cases
---
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
- `LOG-WRITE-14` A stream folder that is a symlink is refused: exit 3 naming the folder, and nothing is written behind the link.
  design §4.3 is silent on symlinks; the S1.3 review ruling: the store may be a symlink, but no symlink below it is followed, by write or by prune (LOG-PRUNE-15, LOG-PRUNE-17). Run: memex log alias where .memex/logs/alias links to a sibling folder.
- `LOG-WRITE-15` A folder already at an entry's target name counts as taken: the entry is written with the `-2` suffix, and the folder and its contents are left as they were.
  design §4.4 Atomic writes: a name that exists in any form is taken, since Windows reports a folder there differently. Run: memex log impl over a stream whose folder holds a folder named like the entry's file.
- `LOG-WRITE-16` A stream name of 40 characters is accepted, and one of 41 exits 2 with `invalid stream name` and writes nothing.
  design §3.4 Stream name: `[a-z0-9][a-z0-9-]{0,39}`, so 40 characters at most. Run: memex log with a 40-character stream, then with a 41-character one.
- `LOG-WRITE-17` Plain stdout is exactly the ref and one newline, with nothing after it.
  design §3.4 Output: stdout is exactly the ref on one line, so `ref=$(memex log …)` works. Run: memex log impl with a one-title body; the golden has no extra row and no `no newline at end` mark.

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

- `LOG-BODY-01` A body of 16385 stored bytes, one over 16 KiB, exits 2 with `log body is over 16 KiB: logs are findings: cite artifacts by path` and writes nothing.
  design §3.4 Body (measured as stored) and §8 item 5. Run: memex log impl with a stdin_file of 16385 LF bytes, which store as read.
- `LOG-BODY-02` A body whose title is empty, such as one line of only `#` marks, exits 2 with `log body has no title line` and writes nothing.
  design §3.4 Body; PLAN §1 fix 7. Run: memex log impl with stdin `#`.
- `LOG-BODY-03` A body of exactly 16384 stored bytes is accepted, and its ref is printed with exit 0.
  design §3.4 Body: the maximum is 16 KiB, measured as stored. Run: memex log impl with a stdin_file of 16384 LF bytes; the golden holds the ref only, since the file is too big to dump.
- `LOG-BODY-04` A body of only spaces, tabs and newlines exits 2 with `log body is empty` and writes nothing.
  design §3.4 Body: required and non-empty, blank counts as empty. Run: memex log impl with stdin of blank lines.
- `LOG-BODY-05` A body whose only text after its frontmatter is blank lines exits 2 with `log body has no title line` and writes nothing.
  design §3.4 Body: the title is the first non-blank line after any frontmatter. Run: memex log impl with a `---` block, then blank lines.
- `LOG-BODY-06` A UTF-8 BOM at the start of a body is dropped, so the agent frontmatter after it still merges, the title is read after it, and the stored entry starts with memex's `---` and has no BOM.
  design §4.4 Text: a leading UTF-8 BOM is dropped on read. Run: memex log impl --agent cook-p1 with a stdin_file that starts with a BOM, then a `status: done` block, then `# BOM title`.
- `LOG-BODY-07` A CRLF body of 16549 bytes, whose LF form is exactly 16384 bytes, is accepted, and its ref is printed with exit 0.
  design §3.4 Body: the limit is measured after CRLF becomes LF. Run: memex log impl with a stdin_file of 16549 CRLF bytes; the golden holds the ref only.
- `LOG-BODY-08` A body of 16384 bytes that ends in the two bytes E2 82, which decode lossily to one U+FFFD and make the stored body 16385 bytes, exits 2 with `log body is over 16 KiB: logs are findings: cite artifacts by path` and writes nothing.
  design §3.4 Body: the limit is measured on the stored body, after lossy decoding. Run: memex log impl with a stdin_file of 16384 bytes that ends in E2 82.
- `LOG-BODY-09` Leading blank lines are skipped before the title, so the ref takes its slug from the first line with text, and the stored file keeps the blank lines, exit 0.
  design §3.4 Body: the title is the first non-blank line after any frontmatter; §4.3: the body is stored as given. Run: memex log impl with a body of two blank lines, then `# Title after blanks`.

## SLUG

- `LOG-SLUG-01` A title of only non-ASCII letters gets the slug `entry`, and a 100-character title gets a 60-character slug that does not end in a dash.
  design §3.4 Body: the slug is lowercase ASCII alphanumerics, each run of other characters one dash, trimmed, at most 60 characters, or `entry` when empty; §4.3 Log files. Run: memex log impl with `# 日本語の題名`, then with `# Title` and 94 more capital letters, 100 characters in all.

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
- `LOG-FRONT-07` An agent key that memex owns, spelled with a capital, `To:`, exits 2 with `frontmatter key 'To' is set by memex` and writes nothing.
  design §3.4 Extra fields: memex's keys compare case-insensitively. Run: memex log impl with `To: [victim]` in the body's frontmatter.
- `LOG-FRONT-08` An agent key that memex owns, spelled in capitals, `AGENT:`, exits 2 with `frontmatter key 'AGENT' is set by memex` and writes nothing.
  design §3.4 Extra fields. Run: memex log impl with `AGENT: evil` in the body's frontmatter.
- `LOG-FRONT-09` An agent key that memex owns, spelled with a capital and a plural, `Tags:`, exits 2 with `frontmatter key 'Tags' is set by memex` and writes nothing.
  design §3.4 Extra fields. Run: memex log impl with `Tags: [show]` in the body's frontmatter.
- `LOG-FRONT-10` An agent key that memex owns, spelled in mixed case, `bRanch:`, exits 2 with `frontmatter key 'bRanch' is set by memex` and writes nothing.
  design §3.4 Extra fields. Run: memex log impl with `bRanch: main` in the body's frontmatter.
- `LOG-FRONT-11` Two agent keys that differ only in case, `foo:` and then `Foo:`, exit 2 with `frontmatter key 'Foo' is repeated` and write nothing.
  design §3.4 Extra fields: a repeated key compares case-insensitively. Run: memex log impl with `foo: 1` then `Foo: 2` in the body's frontmatter.
- `LOG-FRONT-12` A lone carriage return inside an agent frontmatter value exits 2 with `frontmatter value has a carriage return` and writes nothing.
  design §4.4 Text accepts CRLF only, and design §3.4 does not yet state this refusal; it is the S5.2 ruling. Run: memex log impl with a block line `status: ok`, then a lone CR, then `To: [victim]` on that line.

## STREAM

- `LOG-STREAM-01` A reserved stream name, `readmes`, `headers`, `cases` or `logs`, exits 2 with `stream name '<name>' is reserved` and writes nothing.
  design §3.4 Stream name and §3.1 built-in collections. Run: memex log logs, then readmes, headers and cases.
- `LOG-STREAM-02` A stream name whose `.memex/collections/<name>.json` manifest exists exits 2 with `stream name '<name>' is a collection` and writes nothing.
  design §3.4 Stream name; §4.2 manifests. Run: memex log rs-notes with that manifest present, then memex log rs, which is accepted because only the exact file name counts.
- `LOG-STREAM-03` From a linked worktree, a manifest that exists only in the main checkout's `.memex/collections` also makes its name a collection.
  design §4.1 item 3 (the store owner is the main checkout) and §3.4 Stream name. Run: memex log rs-notes from a linked worktree, where `.memex/.gitignore` keeps the manifest out of git.
- `LOG-STREAM-04` A Windows device name, `nul` or `com1`, exits 2 with `stream name '<name>' is a Windows device name` and writes nothing, on every OS.
  design §3.4 Stream name: the device names are refused on every OS so a store stays portable. Run: memex log nul, then memex log com1.
- `LOG-STREAM-05` A manifest name is matched by its exact file name: with `Notes.json` present, `memex log notes` succeeds on every OS, and with `rs-notes.json` present, `memex log rs-notes` exits 2 with `stream name 'rs-notes' is a collection`.
  design §3.4 Stream name (matched against the listed file names, exactly, never by probing) and §4.4 Name matching. Run: memex log notes, then memex log rs-notes, with both manifests present.
- `LOG-STREAM-06` The Windows device names `con`, `prn`, `aux`, `lpt0` and `com0` each exit 2 with `is a Windows device name` and write nothing, on every OS.
  design §3.4 Stream name: the device names are refused on every OS so a store stays portable. Run: memex log con, prn, aux, lpt0 and com0, each with a body.

## READ

- `LOG-READ-01` A dot-named `.md` file in a stream folder is read by no read verb: search, a stream's roster and info all leave it out, and show finds nothing under its name, though the entry beside it is listed.
  design §4.3 (readers ignore dotfiles, so a partial entry is never visible) and §3.5 Streams. Run: memex search widget, then memex search --in impl, then memex info, then memex show impl:.20261013-115500-widget-draft.
- `LOG-READ-02` A stream folder that cannot be read is one stderr warning, `memex: warning: cannot read <path> (PermissionDenied)`, and its entries are not loaded, while `show .`, `search` and `info` still exit 0.
  design §4.2 (an unreadable stream folder is a warning that is skipped) and §4.4 Errors. Run: memex show ., memex search earlier and memex info, with .memex/logs/impl at mode 000.

## PRUNE

- `LOG-PRUNE-19` With keep_logs 3d, `search`, `show` and `info` leave an entry 4 days old in place: it is still listed, shown and counted, and its file is still in the store, since pruning happens only on write.
  design §4.3 Cleanup (pruning happens only on write, and reads never delete). Run: memex search widget, then memex show impl:20261009-110000-widget-old, then memex info, with the store snapshot after.
- `LOG-PRUNE-01` With no config.json, a write deletes an entry older than 3 days by its filename stamp, keeps one 2 days old, and keeps the new entry.
  design §4.3 Cleanup and §4.1 item 4 (the default 3d); PLAN §1 fix 6. Run: memex log impl over a 4-day-old and a 2-day-old entry, with no config.json.
- `LOG-PRUNE-02` With a config.json that has no keep_logs key, the default 3d applies: an entry exactly 3 days old stays, one second older is deleted, and a future-stamped entry stays.
  design §4.1 item 4 (a missing key means 3d) and §4.3 (older than keep_logs). Run: memex log impl with config `{}` over entries at 3 days, at 3 days and one second, and 7 days ahead.
- `LOG-PRUNE-03` keep_logs `90m` deletes an entry older than 90 minutes and keeps one younger.
  design §4.3 (keep_logs takes Nm). Run: memex log impl with keep_logs 90m over entries 91 and 89 minutes old.
- `LOG-PRUNE-04` keep_logs `36h` deletes an entry older than 36 hours and keeps one younger.
  design §4.3 (keep_logs takes Nh). Run: memex log impl with keep_logs 36h over entries 37 and 35 hours old.
- `LOG-PRUNE-05` keep_logs `2w` deletes an entry older than two weeks and keeps one younger.
  design §4.3 (keep_logs takes Nw). Run: memex log impl with keep_logs 2w over entries 15 and 13 days old.
- `LOG-PRUNE-06` keep_logs `forever` deletes nothing, however old the entry.
  design §4.3 ("forever" disables pruning). Run: memex log impl with keep_logs forever over an entry stamped 2020-01-01.
- `LOG-PRUNE-07` An invalid keep_logs, such as `soon`, prints one warning on stderr that names the config file and the value, and deletes nothing.
  design §4.3 (a warning and no pruning); PLAN §1 fix 6. Run: memex log impl with keep_logs soon over a 4-day-old entry.
- `LOG-PRUNE-08` A config.json that is not valid JSON prints its warning and deletes nothing, so the 3-day default does not apply.
  design §4.1 item 4 and §4.3 (never delete on config it can't read); PLAN §1 fix 6. Run: memex log impl with config.json `{not json` over a 4-day-old entry.
- `LOG-PRUNE-09` A config.json with an unknown key prints its warning and deletes nothing.
  design §4.1 item 4 and §4.3; PLAN §1 fix 6. Run: memex log impl with config.json `{"keep_log": "3d"}` over a 4-day-old entry.
- `LOG-PRUNE-10` Only regular files named `YYYYMMDD-HHMMSS-*.md` directly inside a stream folder are deleted: other names, dotfiles, files in subfolders and dot-named stream folders survive.
  design §4.3 ("only files matching"); PLAN §1 fix 6. Run: memex log impl over old entries of each kind, beside one old entry that is deleted.
- `LOG-PRUNE-11` A name whose stamp is not a real UTC date-time, such as month 13, February 30 or hour 25, survives, while a real leap day is deleted like any old entry.
  design §4.3 and §3.6 (UTC); PLAN §1 fix 6. Run: memex log impl over old names with those stamps, and over 20240229-120000.
- `LOG-PRUNE-12` A stream folder that prune empties is removed, and prune reaches streams other than the one written.
  design §4.3 (stream folders emptied by a prune are removed); PLAN §1 fix 6. Run: memex log impl over an old entry alone in stream gone, and an old entry in stream other beside a recent one.
- `LOG-PRUNE-13` A stream folder that was already empty, and that prune did not empty, is left alone.
  design §4.3 (only folders a prune emptied are removed). Run: memex log impl beside an empty folder idle, which setup makes with git clean.
- `LOG-PRUNE-14` A symlink named like a log entry is never deleted, and the file it points to is untouched.
  design §4.3 is silent on symlinks; PLAN §1 fix 6 deletes only log entries, and the S1.3 ruling is regular files only. Run: memex log impl over an old symlink into a sibling folder.
- `LOG-PRUNE-15` A symlinked stream folder is never followed, so the old entries behind it survive.
  design §4.3 is silent on symlinks; PLAN §1 fix 6 and the S1.3 ruling as in LOG-PRUNE-14. Run: memex log impl beside a stream folder symlink into a sibling folder that holds an old entry.
- `LOG-PRUNE-16` A config.json that is a dangling symlink prints a warning that it cannot be read and deletes nothing, so the 3-day default does not apply.
  design §4.1 item 4 and §4.3 (never delete on config it can't read); PLAN §1 fix 6. Run: memex log impl with .memex/config.json a symlink to a missing file, over a 4-day-old entry.
- `LOG-PRUNE-17` A store that is itself a symlink is followed: the entry is written behind it, and the old entries there are pruned.
  design §4.3 is silent on symlinks; the S1.3 review ruling as in LOG-WRITE-14. Run: memex log impl with .memex/logs a symlink to a sibling folder that holds a 4-day-old and a 2-day-old entry.
- `LOG-PRUNE-18` A config.json that is a JSON array, `["1h"]`, prints one warning that it is not a JSON object and deletes nothing, so the array is not read by position.
  design §4.1 item 4 and §4.3 (never delete on config it can't read). Run: memex log impl with config.json `["1h"]` over a 4-day-old entry.
- `LOG-PRUNE-20` A prune delete that the OS refuses, an expired entry in a stream folder at mode 555 while `log` writes to another stream, prints one warning that names the entry, leaves it in place, and the `log` exits 0.
  design §4.4 Atomic writes (a delete the OS refuses is a warning, never an error) and §4.3 Cleanup. Run: memex log impl with a fresh body, over a 4-day-old entry in stream old at mode 555.


