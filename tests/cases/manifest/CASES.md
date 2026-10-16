---
memex: cases
---
# Manifest cases

## LOAD

- `MAN-LOAD-01` A valid folder manifest loads, so `show notes:alpha` prints its doc, and a non-JSON file, a dot-named manifest and a root folder that does not exist raise no warning, exit 0.
  design §4.2 manifests and keys; §4.2 Invalid manifests. Run: memex show notes:alpha. The fixture has `notes.txt` and `dot_draft.json` beside `notes.json`, and `notes.json` names the roots `notes` and `missing` with the marker `index.md`, so its doc is `notes/alpha/index.md`.
- `MAN-LOAD-02` A manifest whose `name` is not its file name is skipped with one warning, which prints once for a run with two refs, and both refs are not found, exit 1.
  design §4.2 `name` (must equal the file stem) and Invalid manifests. Run: memex show journal:alpha notes:alpha. The fixture's `notes.json` names itself `journal`.
- `MAN-LOAD-03` A manifest named after a reserved collection, `readmes`, `headers` or `logs`, is refused with a warning and skipped, so its refs are not found, exit 1, while a manifest named `cases` loads, so its doc `alpha` is found.
  design §3.1 Built-in collections and §4.2 `name` (reserved names rejected); PLAN §13 R14. Run: memex show `cases:<id>` and `logs:<id>` for the id alpha.
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
  design §4.2 `kind` (folder or json), `roots` (root-relative folders) and `include` (globs, json only). Run: memex show badglob:alpha badkind:alpha outside:alpha, with warnings in file-name order. The unknown kind is `table`, and the bad glob sits in a json manifest, since `include` is json only.
- `MAN-LOAD-11` A key that its kind does not use, `at` on a folder manifest, gives the warning `key 'at' does not apply to kind 'folder'` and skips the manifest, exit 1.
  design §4.2 Kinds column (`at` applies to json only) and Invalid manifests. Run: memex show notes:alpha.
- `MAN-LOAD-12` A json manifest with no `id` gives a warning and is skipped, so its ref is not found, exit 1.
  design §4.2 `id` (json: entry field, required). Run: memex show rows:one.
- `MAN-LOAD-13` A manifest's `weights` raise its fields' ranking, so with `title` at 12 a doc whose title holds the word ranks above a body-only match that ranks first under the default weights.
  design §4.2 `weights` (a per-field ranking boost, with the defaults id 4, title 3, body 1). Run: memex search meridian --in plain, then memex search meridian --in raised.
- `MAN-LOAD-14` A manifest with `kind: markdown` is skipped with the warning `kind 'markdown' is gone; add 'memex: notes' to its files' frontmatter instead`, so the collections line names no `notes`, and its Markdown file with no `memex:` key is no doc, exit 0.
  design §4.2 `kind: markdown` (the manifest is skipped with that warning). Run: memex info. The fixture's `notes.json` says `kind: markdown`, and `notes/alpha.md` has no frontmatter.

The folder and json kinds of design §4.2 are cased as `MANK-*` in the [manifest-kinds](../manifest-kinds/CASES.md) family. The Markdown kind is gone: a Markdown collection is a `memex:` frontmatter collection, pinned by the [collection](../collection/CASES.md) family.

## IO

- `MAN-IO-01` A manifest that cannot be read is one stderr warning, `memex: warning: cannot read <path> (PermissionDenied)`, its collection is left out of `info`, and the other collection still loads, so `info` and `show notes:alpha` exit 0.
  design §4.2 Unreadable files (the form, applied to a manifest file). Run: memex info, then memex show notes:alpha. The fixture's `locked.json` is chmod 000 beside `notes.json`, a folder manifest, which still loads.
