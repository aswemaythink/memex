# Manifest kind cases

## FOLDER

- `MANK-FOLDER-01` (planned) Each folder that holds the marker file is one doc, labelled `<name>:<id>`, and a folder without the marker gives no doc.
  design §4.2 folder: each folder containing `marker` is one doc. Scenario: a marker `case.json` in two folders and none in a third.
- `MANK-FOLDER-02` (planned) A JSON marker's top-level scalars are the doc's fields, and its nested objects, arrays and nulls are not.
  design §4.2 folder: the marker's fields come from top-level JSON scalars. Scenario: a marker holding a string, a number, a boolean, an object, an array and a null.
- `MANK-FOLDER-03` (planned) A Markdown marker's frontmatter gives the doc's fields, and its body starts the doc's text.
  design §4.2 folder and frontmatter: a `.md` marker's frontmatter gives the fields. The body starting the text is not in the design; it is the reading the unit tests take, and a ruling is pending. Scenario: `case.md` with frontmatter and a heading.
- `MANK-FOLDER-04` (planned) A `merge` file that is Markdown adds its full text to the doc's text.
  design §4.2 folder: each merge file adds its full text if it is Markdown. Scenario: `merge` names a `README.md` in the marker's folder.
- `MANK-FOLDER-05` (planned) A `merge` file that is code adds only its header, by the header rule of design §2.1, and never its body.
  design §4.2 folder: only its header if it is code, so memex never reads code bodies. Scenario: `merge` names a `.rs` file whose body holds a function that must not appear.
- `MANK-FOLDER-06` (planned) With no `id` key the doc's id is its folder's name, and with `id` naming a marker field the id is that field's text, so a marker without that field gives no doc.
  design §4.2 `id`: folder is the marker field or the folder name (default). The no-doc reading for a missing field is not in the design; it is the one the unit tests take. Scenario: a folder `NAME-009` with an empty marker object, and a marker without the named field.
- `MANK-FOLDER-07` (planned) With `title` naming a field, that field's first sentence is the one-liner, and with no `title` key the one-liner is the first `#` heading of the text, or else its first line.
  design §4.2 `title` (default: first `#` heading or first line) and §2 one-liners (the first sentence). Scenario: a `README.md` merge whose first line holds two sentences.
- `MANK-FOLDER-08` (planned) A root may point into a hidden folder, and the hidden entries below the root are skipped.
  design §4.1 item 2: manifest roots may point into hidden folders, and the skip list covers hidden entries. Scenario: root `.agents/cases` with a hidden child folder that holds a marker.
- `MANK-FOLDER-09` (planned) A marker that is not JSON gives no doc, and the other folders still give theirs.
  design §4.2 invalid manifests: bad JSON is a warning and skip. The design names warnings only for manifests, so no warning is printed for a marker; that is pending a ruling. Scenario: one folder with a broken `case.json` and one with a valid one.

## JSON

- `MANK-JSON-01` (planned) Each object of the array at `at`, in every file that `include` matches under the roots, is one doc labelled `<name>:<id>`.
  design §4.2 json: each entry of the array at `at`, in every matched file, is one doc. Scenario: two `.json` files under a root, each holding an array of entries.
- `MANK-JSON-02` (planned) `at` is a JSON Pointer to the array, and with no `at` the whole document is the array.
  design §4.2 `at`: a JSON Pointer to the entries array, default document root. Scenario: `at` set to `/cases` on one file, and no `at` on a file that is a bare array.
- `MANK-JSON-03` (planned) An entry's scalar values are its fields, and its string values are its indexed text.
  design §4.2 json: scalars become fields and strings are indexed as text. The design does not say what nested values do; here they are left out of both. Scenario: an entry with a string, a number, a boolean, an array and an object.
- `MANK-JSON-04` (planned) An entry without a non-empty `id` field gives no doc, since `id` is required for json.
  design §4.2 `id`: json entry field (required). Scenario: entries with no `id`, an empty `id`, and a bare string.
- `MANK-JSON-05` (planned) With `title` naming a field, that field's first sentence is the one-liner, and with no `title` key the one-liner is the first `#` heading of the text, or else its first line.
  design §4.2 `title`. Scenario: `title: claim` on entries whose `claim` field holds two sentences.
- `MANK-JSON-06` (planned) Only the files that `include` matches are read, so a file that no glob matches gives no docs.
  design §4.2 `include`: globs under roots. The design gives no default `include` for json, so a manifest without one matches no file; that is pending a ruling. Scenario: `**/*.json` over a root with a nested file and a `.txt` file holding an entry.
- `MANK-JSON-07` (planned) A file that is not JSON, or whose `at` does not reach an array, gives no docs, and the other matched files still give theirs.
  design §4.2 invalid manifests: bad JSON is skipped and never fails the run. Scenario: one broken file, one whose `at` reaches an object, and one valid file.
- `MANK-JSON-08` (planned) A root may point into a hidden folder, as a folder root may.
  design §4.1 item 2: manifest roots may point into hidden folders. Scenario: root `.agents/data` holding `x.json`.
