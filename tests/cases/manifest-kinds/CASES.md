---
memex: cases
---
# Manifest kind cases

## FOLDER

- `MANK-FOLDER-01` Each folder that holds the marker file is one doc, labelled `<name>:<id>`, and a folder without the marker gives no doc, exit 1 for its ref.
  design §4.2 folder: each folder containing `marker` is one doc. Scenario: a marker `case.json` in two folders and none in a third. Run: memex show units:alpha units:beta units:gamma, then with --json. The manifest has no `id` key, so the id is the folder name.
- `MANK-FOLDER-02` A JSON marker's top-level scalars are the doc's fields, in the order the file gives them, and its nested objects, arrays and nulls are not.
  design §4.2 folder: the marker's fields come from top-level JSON scalars. Ruling: fields keep the file's key order, which the text fields block shows; the keys of the `--json` fields object are sorted by the renderer. Scenario: a marker holding a string, a number, a boolean, an object, an array and a null. Run: memex show scalar:SCALAR-1, then with --json.
- `MANK-FOLDER-03` A Markdown marker's frontmatter gives the doc's fields, and its body starts the doc's text.
  design §4.2 folder and frontmatter: a `.md` marker's frontmatter gives the fields. Ruling: the body starts the text, and the merge files follow it (MANK-FOLDER-04). Scenario: `case.md` with frontmatter and a heading. Run: memex show memo:NOTE-1, then with --json.
- `MANK-FOLDER-04` A `merge` file that is Markdown adds its full text to the doc's text, and the merge files follow the body in name order.
  design §4.2 folder: each merge file adds its full text if it is Markdown. Ruling: merge files follow the body in name order, not in the order the `merge` key names them, and a file that `merge` does not name adds nothing. Scenario: `merge` names `README.md` and `EXTRA.md`, where `EXTRA.md` sorts first, and a `.txt` file it does not name. Run: memex show merged:MERGE-1 --json.
- `MANK-FOLDER-05` A `merge` file that is code adds only its header, by the header rule of design §2.1, and never its body.
  design §4.2 folder: only its header if it is code, so memex never reads code bodies. Scenario: `merge` names a `.rs` file whose body holds a function that must not appear. Run: memex show code:CODE-1 --json.
- `MANK-FOLDER-06` With no `id` key the doc's id is its folder's name, and with `id` naming a marker field the id is that field's text, so a marker without that field gives no doc, exit 1 for its ref.
  design §4.2 `id`: folder is the marker field or the folder name (default). The no-doc reading for a missing field is not in the design; it is the one the golden pins, and a ruling is pending. Scenario: `byname` has no `id` key and the folder `NAME-009`; `byfield` has `id: code`, and its folder `two` has no `code` field. Run: memex show byname:NAME-009 byfield:FIELD-1 byfield:two --json.
- `MANK-FOLDER-07` With `title` naming a field, that field's whole text is the one-liner, and with no `title` key the one-liner is the first `#` heading outside a code fence of the text, or else its first line, with no sentence cut.
  design §4.2 `title` (one rule for every kind, with no first-sentence cut). Scenario: `titled` names `claim` as its `title`; `plain` merges a `README.md` whose first line holds two sentences, and another whose text has a heading after a first line. Run: memex show titled:TITLE-1 plain:PLAIN-1 plain:PLAIN-2 --json.
- `MANK-FOLDER-08` A root may point into a hidden folder, and the hidden entries below the root are skipped, exit 1 for the skipped one.
  design §4.1 item 2: manifest roots may point into hidden folders, and the skip list covers hidden entries. Scenario: root `.agents/cases` with a hidden child folder that holds a marker. Run: memex show agents:HID-01 agents:.hidden --json.
- `MANK-FOLDER-09` A `case.json` marker that does not parse as JSON gives no doc, and the other folders still give theirs, exit 1 for the broken one.
  design §4.2 invalid manifests: bad JSON is a warning and skip. The golden pins the warning that a broken marker prints on stderr; the design names warnings for manifests only, so that is pending a ruling. Scenario: one folder with a broken `case.json` and one with a valid one. Run: memex show broken:BAD-1 broken:GOOD-1 --json.

## JSON

- `MANK-JSON-01` Each object of the array at `at`, in every file that `include` matches under the roots, is one doc labelled `<name>:<id>`.
  design §4.2 json: each entry of the array at `at`, in every matched file, is one doc. Scenario: `include` is `**/*.json`, and two files under the root, one of them nested, each hold an `items` array. Run: memex show rows:ONE-1 rows:ONE-2 rows:TWO-1 --json.
- `MANK-JSON-02` `at` is a JSON Pointer to the array, and with no `at` the whole document is the array.
  design §4.2 `at`: a JSON Pointer to the entries array, default document root. Scenario: `pointer` has `at` `/data/cases` over a nested object, and `bare` has no `at` over a file whose document is the array. Run: memex show pointer:PTR-1 bare:BARE-1 bare:BARE-2 --json.
- `MANK-JSON-03` An entry's scalar values are its fields, in the order the file gives them, and its string values are its indexed text.
  design §4.2 json: scalars become fields and strings are indexed as text. The design does not say what nested values do; here they are left out of both fields and text, and so are nulls. Ruling: fields keep the file's key order. Scenario: an entry with a string, a number, a boolean, an array, an object and a null. Run: memex show scalars:SCALAR-1, then with --json.
- `MANK-JSON-04` An entry without a non-empty `id` field gives no doc, since `id` is required for json, so the empty-id label is not found, exit 1.
  design §4.2 `id`: json entry field (required). Scenario: entries with no `id`, an empty `id` and a bare string, beside one entry with an id. The entries with no id cannot be named, and `info` and `search` are not built yet, so only the empty-id label and the valid entry are observable. Run: memex show ids:OK-1 ids:, then with --json.
- `MANK-JSON-05` With `title` naming a field, that field's whole text is the one-liner, and with no `title` key the one-liner is the first `#` heading outside a code fence of the text, or else its first line, with no sentence cut.
  design §4.2 `title` (one rule for every kind, with no first-sentence cut). Scenario: `titled` names `claim` as its `title`, on an entry whose `claim` holds two sentences; `plain` has no `title`, and its entries give a heading, or else a first line of two sentences. Run: memex show titled:T-1 plain:P-1 plain:P-2 --json.
- `MANK-JSON-06` Only the files that `include` matches are read, so a file that no glob matches gives no docs. A glob with no slash matches at any depth, and a glob with a leading slash is anchored at its root. A json manifest with no `include` is refused.
  design §4.2 `include`: globs under roots; the glob rules are those of `ignore`'s override matcher. Ruling: a json manifest with no `include` is an invalid manifest, so it gives the warning `kind 'json' needs key 'include'` and is skipped, as a missing `id` is. Scenario: `globs` (`*.json`) and `anchor` (`/*.json`) over one root that holds a top-level file, a nested file and a `.txt` file; `noinc` has no `include`. Run: memex show globs:TOP-1 globs:NEST-1 globs:TXT-1 anchor:TOP-1 anchor:NEST-1 noinc:TOP-1 --json, exit 1.
- `MANK-JSON-07` A file that is not JSON, or whose `at` does not reach an array, gives no docs, and the other matched files still give theirs.
  design §4.2 invalid manifests: bad JSON is skipped and never fails the run. The golden pins the warning that a broken data file prints on stderr; the design names warnings for manifests only, so that is pending a ruling. Scenario: one broken file, one whose `at` reaches an object, and one valid file. Run: memex show mixed:GOOD-1 mixed:OBJ-1 --json, exit 1.
- `MANK-JSON-08` A root may point into a hidden folder, as a folder root may.
  design §4.1 item 2: manifest roots may point into hidden folders. Scenario: root `.agents/data` holding `x.json`. Run: memex show hiddata:HID-1 --json.

## TITLE

- `MANK-TITLE-01` A folder or json doc's default one-liner skips a `#` line inside a code fence, and takes the first heading outside it, as a markdown doc does.
  design §4.2 `title` (the first ATX heading outside fenced code) and §2.3 (the one fence definition). Scenario: `fenced` is a json manifest whose entry's text has a fenced `# Fenced` above `# Real heading`, and `fencedfolder` is a folder manifest whose `case.md` has the same fence above its heading. Run: memex show fenced:FENCE-1 fencedfolder:FENCE-2 --json.
- `MANK-TITLE-02` A run of seven `#` marks is text, so its line is the one-liner, and a run of six marks is a heading, for json and folder docs.
  design §4.2 `title` (one to six `#` marks, then a space or tab). Scenario: `marks` has an entry whose first line is `####### Seven marks` and an entry whose text holds `###### Six marks` after a lead line; `marksfolder` has a `case.md` whose first line is the seven-mark line. Run: memex show marks:SEVEN-1 marks:SIX-1 marksfolder:SEVEN-2 --json.
- `MANK-TITLE-03` An empty `title` field gives the default one-liner, and a `title` field with text gives its whole text, for json and folder docs.
  design §4.2 `title` (an empty title field falls back to the default). Scenario: `emptytitle` names `claim` as its title; `EMPTY-1` has an empty `claim` and a note whose second line is a heading, and `FULL-1` has a `claim` of two sentences; `emptyfolder` names `claim` too, and its `case.md` has an empty `claim:` field and a heading. Run: memex show emptytitle:EMPTY-1 emptytitle:FULL-1 emptyfolder:EMPTY-2 --json.

## MERGE

- `MANK-MERGE-01` A folder manifest's `merge` file that cannot be read gives its `cannot read` warning and adds nothing, and the doc still loads, for a Markdown merge file and for a code merge file alike, exit 0, unix only.
  design §4.2 Unreadable files: a folder manifest's `merge` file that can't be read gets the same warning. Scenario: `mergewarn` merges `README.md`, `NOTES.md` and `spec.rs` into `MERGE-1`; `NOTES.md` and `spec.rs` are chmod 000 (the case sets `platforms` to unix), and `README.md` reads. Run: memex show mergewarn:MERGE-1 --json.

## ROOTS

- `MANK-ROOTS-01` A manifest root that is a symlink to a folder outside the repo is left out with the warning `<collection>: root <root> leads outside the repo; skipped`, while a symlink to a folder inside the repo still reads, and the outside doc is not found, exit 1, unix only.
  design §4.2 Unreadable files: a manifest root that is a symlink resolving outside the repo root is skipped with that warning. Scenario: `links` names the roots `linked`, a symlink to `../outside` beside the repo, and `shortcut`, a symlink to `units`; `links.patch` makes both symlinks with `git apply` in setup, since a fixture cannot hold a symlink, and that patch gives plain files on Windows. Run: memex show links:SHORT-1 links:Outside --json, then memex info.

## WT

- `MANK-WT-01` In a linked worktree, a json manifest's gitignored root that only the main checkout holds is read from the main checkout: `search` finds its docs with their usual labels, exit 0, and stderr prints once `<collection>: root <root> is not in this worktree; read from the main checkout <main-root>/<root>`.
  design §4.2 Roots in a linked worktree. Scenario: `.gitignore` lists `corpus/`; `.memex/collections/corpus.json` is a json manifest over `corpus` with `include` `["corpus.json"]`; `corpus/corpus.json` holds two docs and exists only in repo/ (git commit leaves it out). Setup: git worktree add ../wt -b feat. Run from wt: memex search slot recipe --in corpus, then memex search slot recipe --in corpus --json, then memex info.
- `MANK-WT-02` In a linked worktree, a folder manifest's missing root is read from the main checkout too, with its `merge` files, so `show` prints the borrowed doc and its merged text, exit 0, with the same warning.
  design §4.2 Roots in a linked worktree. Scenario: `.gitignore` lists `built/`; `.memex/collections/built.json` is a folder manifest over `built` with marker `case.json` and merge `["README.md"]`; `built/BUILT-1/` holds `case.json` and `README.md` in repo/ only. Setup: git worktree add ../wt -b feat. Run from wt: memex show built:BUILT-1.
- `MANK-WT-03` The guard holds in a linked worktree: a root that the worktree holds as a symlink to a folder outside both checkouts is skipped with `<collection>: root <root> leads outside the repo; skipped`, and a root missing in the worktree whose main-checkout path is a symlink to a folder outside the main checkout is skipped with the same warning, so neither outside doc is found, exit 1, unix only.
  design §4.2 Roots in a linked worktree and Unreadable files. Scenario: a sibling folder `outside/` beside repo/ holds a marker doc; `linked` is a committed symlink to it, and `ghost` is a gitignored symlink to it present only in repo/; both are roots of one folder manifest that also names a plain committed root `units`. Run from wt: memex show far:UNIT-1 far:Outside --json, then memex info.
- `MANK-WT-04` A root the worktree holds is the worktree's own, even when the main checkout's copy differs, with no warning; a root missing in both checkouts reads nothing and warns nothing; and in the main checkout a root is never borrowed, exit 0.
  design §4.2 Roots in a linked worktree. Scenario: roots `own` (repo/ gets an untracked extra doc, `own/OWN-2`, by git apply after the worktree is made, so the two checkouts differ) and `nowhere` (in neither). Run from wt: memex search widget --in mine --json; then from repo: memex info.
- `MANK-WT-05` With git missing from PATH, a linked worktree still borrows a missing root from the main checkout, since the counterpart comes from the layout files alone, exit 0, unix only.
  design §4.2 Roots in a linked worktree and §4.4 git. Scenario: the MANK-WT-01 tree. Run from wt with PATH=/nonexistent: memex search slot recipe --in corpus.

## CITE

- `MANK-CITE-01` A folder doc's relative citations resolve from the folder that holds its marker: the `../SPEC.md` link and the `../src/lib.rs` path in its merged README cite the files in the folder above the case folder, and each cited file lists the doc under cited by, exit 0.
  design §2.3: a link and a path token resolve from the citing doc's folder, then the root; design §4.2 folder: each folder containing `marker` is one doc. Ruling: a folder doc's folder is the folder that holds its marker, and its merged files resolve from there. Scenario: `cases` is a folder manifest over `units`, with marker `case.json` and merge `README.md`; `units/ATM-1` holds the marker and that README, and `units` holds `SPEC.md` and `src/lib.rs`. Run: memex show ATM-1, then memex show units/SPEC.md, then memex show units/src/lib.rs, then memex show ATM-1 --json, whose path stays null.
- `MANK-CITE-02` A json doc's relative path resolves from the folder of its JSON file first, then the root: `docs/x.md` in `data/cases.json` cites `data/docs/x.md` where that file sits, and `notes/y.md` cites the root's file, since the JSON file's folder holds none, exit 0.
  design §2.3: a path token resolves from the citing doc's folder, then the root. Ruling: a json doc's folder is the folder of the JSON file it came from, since design §4.2 names no folder for a manifest doc. Scenario: `rows` is a json manifest over `data`, with `include` `cases.json` and `id` `name`; `data/docs/x.md`, `docs/x.md` and `notes/y.md` exist. Run: memex show rows:JS-1 rows:JS-2, then memex show data/docs/x.md docs/x.md notes/y.md.
