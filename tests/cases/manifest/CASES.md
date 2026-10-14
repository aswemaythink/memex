# Manifest cases

## LOAD

- `MAN-LOAD-01` A valid markdown manifest loads, so `show notes:alpha` prints its doc, and a non-JSON file, a dot-named manifest and a root folder that does not exist raise no warning, exit 0.
  design §4.2 manifests and keys; §4.2 Invalid manifests. Run: memex show notes:alpha. The fixture has `notes.txt` and `dot_draft.json` beside `notes.json`, and `notes.json` names the roots `notes` and `missing`.
- `MAN-LOAD-02` A manifest whose `name` is not its file name is skipped with one warning, which prints once for a run with two refs, and both refs are not found, exit 1.
  design §4.2 `name` (must equal the file stem) and Invalid manifests. Run: memex show journal:alpha notes:alpha. The fixture's `notes.json` names itself `journal`.
- `MAN-LOAD-03` A manifest named after a reserved collection, `cases` or `logs`, is refused with a warning and skipped, so its refs are not found, exit 1.
  design §3.1 Built-in collections and §4.2 `name` (reserved names rejected). Run: memex show cases:alpha logs:alpha.
- `MAN-LOAD-04` A manifest with a key the table does not define gives the warning `unknown key 'colour'` and is skipped, exit 1.
  design §4.2 Invalid manifests (an unknown key is a warning, and the collection is skipped). Run: memex show notes:alpha.
- `MAN-LOAD-05` A manifest that is not valid JSON gives a warning that names the parser's reason and is skipped, exit 1.
  design §4.2 Invalid manifests. Run: memex show notes:alpha. The fixture has a trailing comma in `notes.json`.
- `MAN-LOAD-06` A root in a hidden folder is read, while a hidden entry below it is skipped as in the intent layer, so `hidden:alpha` prints and `hidden:draft` is not found, exit 1.
  design §4.1 item 2 (manifest roots may point into hidden folders) and §4.2 `roots`. Run: memex show hidden:alpha hidden:draft.
- `MAN-LOAD-07` A `weights` object of whole numbers is accepted, so the manifest loads and its doc prints, exit 0.
  design §4.2 `weights` (per-field ranking boost, read by search). Run: memex show notes:alpha, with weights for id, title, body and status.
- `MAN-LOAD-08` A weight that is not a whole number gives the warning `weight 'id' must be a whole number` and skips the manifest, exit 1.
  design §4.2 `weights` and Invalid manifests. Run: memex show notes:alpha, with `"id": "four"`.
- `MAN-LOAD-09` A manifest with no `kind` and a manifest with no `roots` each give a warning that names the missing key, and both are skipped, exit 1.
  design §4.2 `roots` (required) and `kind` (the table's kinds). Run: memex show nokind:alpha noroots:alpha.
- `MAN-LOAD-10` A manifest with an unknown kind, a root that climbs out of the root, or an include glob that does not parse each give one warning and are skipped, exit 1.
  design §4.2 `kind`, `roots` (root-relative folders) and `include` (globs). Run: memex show badglob:alpha badkind:alpha outside:alpha, with warnings in file-name order.
- `MAN-LOAD-11` A key that its kind does not use, `at` on a markdown manifest, gives the warning `key 'at' does not apply to kind 'markdown'` and skips the manifest, exit 1.
  design §4.2 Kinds column (`at` applies to json only) and Invalid manifests. Run: memex show notes:alpha.
- `MAN-LOAD-12` A json manifest with no `id` gives a warning and is skipped, so its ref is not found, exit 1.
  design §4.2 `id` (json: entry field, required). Run: memex show rows:one.

## MD

- `MAN-MD-01` A markdown doc's frontmatter scalars and flow lists are its fields in file order, and its body is its text, in the text and the `--json` output of `show`, exit 0.
  design §4.2 Frontmatter and `markdown`; §3.3 Log or collection doc; §3.6 Doc. Run: memex show notes:alpha, then memex show notes:alpha --json.
- `MAN-MD-02` A `- item` block list is a list field with its quotes removed, shown as `[a, b]` in text and as a JSON array, exit 0.
  design §4.2 Frontmatter (`- item` lists). Run: memex show notes:beta, then memex show notes:beta --json.
- `MAN-MD-03` A nested frontmatter block is kept as its text under its key and is not split into fields, exit 0.
  design §4.2 Frontmatter (nested text is kept as written and cannot be filtered). Run: memex show notes:gamma, then memex show notes:gamma --json.
- `MAN-MD-04` An `id` of `stem`, the default, names a doc by its file stem even when it has a `slug` field, a doc with no frontmatter has no fields, and the slug is not an id, exit 1 for the slug ref.
  design §4.2 `id` (stem is the default) and §3.1 labels. Run: memex show notes:alpha notes:plain, then memex show notes:alpha-slug.
- `MAN-MD-05` An `id` of `path` names a doc by its root-relative path with its folders and extension, so `paths:notes/alpha.md` and `paths:notes/deep/delta.md` are the docs, exit 0.
  design §4.2 `id` (path). Run: memex show paths:notes/alpha.md paths:notes/deep/delta.md --json.
- `MAN-MD-06` An `id` that names a frontmatter field takes that field's value as the id, and a doc without the field keeps its file stem, so `slugs:alpha-slug` and `slugs:beta` both resolve, exit 0.
  design §4.2 `id` (a frontmatter field) and §4.2 Frontmatter (missing fields simply do not score). Run: memex show slugs:alpha-slug slugs:beta.
- `MAN-MD-07` A doc's default title is its first `#` heading outside a code fence, or its first non-blank line when it has no heading, as the `--json` output shows, exit 0.
  design §4.2 `title` (default: first # heading, else first line). Run: memex show notes:fenced notes:plainline --json.
- `MAN-MD-08` A manifest `title` that names a frontmatter field gives the doc its one-liner from that field, and a doc without the field falls back to its default title, exit 0.
  design §4.2 `title` (field for the one-liner). Run: memex show titled:alpha titled:fenced --json.
- `MAN-MD-09` A matched `.json` file is a doc whose top-level string, number and boolean values are its fields in key order, with arrays, objects and nulls left out and no text, exit 0.
  design §4.2 `markdown` (matched `.json` sidecars are docs with their top-level scalars as fields). Run: memex show sidecars:meta, then memex show sidecars:meta --json.
- `MAN-MD-10` A sidecar that is not valid JSON gives a warning and no doc, while the valid sidecar beside it still shows, exit 1.
  design §4.2 Invalid manifests (a warning, and the file is skipped), applied to one file. Run: memex show sidecars:broken sidecars:meta.
- `MAN-MD-11` The include globs choose the files: the default `**/*.md` matches Markdown at any depth but not a `.txt`, a glob with a slash is anchored at its root, and a glob without one matches at any depth, exit 1 for the refs that match nothing.
  design §4.2 `include` (markdown default `**/*.md`); the glob rules are those of `ignore`'s override matcher. Run: memex show notes:delta anchored:delta anywhere:delta notes:readme, then memex show anchored:alpha anywhere:alpha.
- `MAN-MD-12` A UTF-8 byte-order mark before the frontmatter is dropped and CRLF line ends are read, so the fields parse, exit 0.
  design §4.4 Text (a leading BOM is dropped on read; \r\n is accepted in frontmatter). Run: memex show notes:bom.
- `MAN-MD-13` A root of `.` is the root itself, so the manifest reads the whole tree, but the walk skips a `.ignore`d file, `node_modules` and `target`, so the three refs that are not read are not found, exit 1.
  design §4.1 item 2 (the skip list: ignored files, node_modules and target) and §4.2 `roots`. Run: memex show everything:top everything:deep everything:private everything:pkg everything:built.

## FOLDER

- `MAN-FOLDER-01` (planned) A folder that holds the marker file is one doc, whose fields are the marker's top-level scalars, or its frontmatter when the marker is a `.md` file.
  design §4.2 `folder` (the marker file defines a doc; fields come from its scalars or its frontmatter).
- `MAN-FOLDER-02` (planned) Each `merge` file adds its full text when it is Markdown and only its header when it is code, so no code body is read.
  design §4.2 `folder` (`merge` files; code merges header only).
- `MAN-FOLDER-03` (planned) The id of a folder doc is the marker's field named by the manifest's `id`, or the folder name when `id` is not given.
  design §4.2 `id` (folder: marker field or folder name, default).

## JSON

- `MAN-JSON-01` (planned) Each entry of the array at `at` in every matched file is one doc, with its scalars as fields and its strings indexed as text.
  design §4.2 `json`.
- `MAN-JSON-02` (planned) `at` is a JSON Pointer to the entries array, and the document root is the array when `at` is not given.
  design §4.2 `at` (default: document root).
- `MAN-JSON-03` (planned) The id of each entry is the field that the manifest's `id` names, and a json manifest must name one, as MAN-LOAD-12 pins.
  design §4.2 `id` (json: entry field, required).
