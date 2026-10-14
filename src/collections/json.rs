//! Reads a `json` manifest's docs: each object of the array at `at` is one doc (design §4.2).
//! It also holds the helpers that the folder kind shares with it: fields, one-liners, docs and glob matches.
//! Cases: MANK-JSON-01, MANK-JSON-02, MANK-JSON-03, MANK-JSON-04, MANK-JSON-05, MANK-JSON-06, MANK-JSON-07, MANK-JSON-08

use std::fs;
use std::path::Path;

use ignore::overrides::OverrideBuilder;
use serde_json::{Map, Value as Json};

use super::frontmatter::Value;
use super::manifest::Manifest;
use crate::corpus::Doc;
use crate::intent::sentence;
use crate::walk::{self, Entry};

/// The docs of a json manifest: one per object, in each matched file, whose `id` field is text. A manifest with no `id` key
/// gives none, since the id is required.
pub fn docs(manifest: &Manifest, root: &Path) -> Vec<Doc> {
    let Some(key) = manifest.id.as_deref() else {
        return Vec::new();
    };
    let at = manifest.at.as_deref().unwrap_or("");
    let mut docs = Vec::new();
    for dir in &manifest.roots {
        let base = root.join(dir);
        let entries = walk::walk(&base);
        let files: Vec<&Entry> = entries.iter().filter(|entry| !entry.is_dir).collect();
        for file in glob_hits(&base, &manifest.include, &files) {
            for item in items_at(&file.path, at) {
                docs.extend(entry_doc(manifest, key, &item));
            }
        }
    }
    docs
}

/// The items of the array that `at` points to in the JSON file at `path`, a JSON Pointer where empty means the whole document.
/// A file that is not JSON, or whose `at` does not reach an array, has none.
fn items_at(path: &Path, at: &str) -> Vec<Json> {
    let Ok(bytes) = fs::read(path) else {
        return Vec::new();
    };
    let Ok(document) = serde_json::from_slice::<Json>(&bytes) else {
        return Vec::new();
    };
    document
        .pointer(at)
        .and_then(Json::as_array)
        .cloned()
        .unwrap_or_default()
}

/// The doc of one entry: its scalars as fields, its string scalars as the text, and its `id` field as the id. None when the
/// entry is not an object, or its `id` is missing or empty.
fn entry_doc(manifest: &Manifest, key: &str, item: &Json) -> Option<Doc> {
    let object = item.as_object()?;
    let fields = scalar_fields(object);
    let id = field_text(&fields, key).filter(|id| !id.is_empty())?;
    let text = text_of(object);
    Some(doc(manifest, id, text, fields))
}

/// The text of an entry: its string scalars, one per line, in key order.
fn text_of(object: &Map<String, Json>) -> String {
    object
        .values()
        .filter_map(Json::as_str)
        .collect::<Vec<&str>>()
        .join("\n")
}

/// The files among `files` that one of `globs` matches, read relative to `root` with the gitignore rules, so `*.md` matches a
/// name and `**/*.json` matches at any depth. A glob that does not parse matches nothing.
pub(super) fn glob_hits<'a>(root: &Path, globs: &[String], files: &[&'a Entry]) -> Vec<&'a Entry> {
    let mut builder = OverrideBuilder::new(root);
    for glob in globs {
        // A glob that does not parse is skipped, so it matches nothing.
        let _ = builder.add(glob);
    }
    let Ok(overrides) = builder.build() else {
        return Vec::new();
    };
    files
        .iter()
        .copied()
        .filter(|file| overrides.matched(&file.path, false).is_whitelist())
        .collect()
}

/// The fields that an object's top-level scalars give, in key order: strings, numbers and booleans as text. Nulls, arrays
/// and nested objects give none.
pub(super) fn scalar_fields(object: &Map<String, Json>) -> Vec<(String, Value)> {
    object
        .iter()
        .filter_map(|(key, value)| scalar_text(value).map(|text| (key.clone(), Value::Str(text))))
        .collect()
}

/// The text of a JSON scalar, or None for null, an array or an object.
fn scalar_text(value: &Json) -> Option<String> {
    match value {
        Json::String(text) => Some(text.clone()),
        Json::Number(number) => Some(number.to_string()),
        Json::Bool(flag) => Some(flag.to_string()),
        Json::Null | Json::Array(_) | Json::Object(_) => None,
    }
}

/// The text of a field that holds a string, or None when it is missing or holds a list.
pub(super) fn field_text(fields: &[(String, Value)], key: &str) -> Option<String> {
    fields
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, value)| match value {
            Value::Str(text) => Some(text.clone()),
            Value::List(_) => None,
        })
}

/// The doc of a manifest: labelled `<name>:<id>`, with its one-liner from the `title` field or the text's default.
pub(super) fn doc(
    manifest: &Manifest,
    id: String,
    text: String,
    fields: Vec<(String, Value)>,
) -> Doc {
    let title = one_liner(manifest.title.as_deref(), &fields, &text);
    Doc {
        label: format!("{}:{id}", manifest.name),
        collection: manifest.name.clone(),
        id,
        path: None,
        title,
        text,
        fields,
    }
}

/// The one-liner: the text of the field that the manifest's `title` names, else the text's default, as its first sentence.
fn one_liner(title: Option<&str>, fields: &[(String, Value)], text: &str) -> String {
    let line = title
        .and_then(|key| field_text(fields, key))
        .unwrap_or_else(|| default_title(text));
    sentence(&line)
}

/// The default one-liner of a text: its first `#` heading without the hashes, else its first non-blank line.
fn default_title(text: &str) -> String {
    let heading = text
        .lines()
        .map(str::trim_start)
        .find(|line| is_heading(line));
    let line = heading.or_else(|| text.lines().find(|line| !line.trim().is_empty()));
    line.unwrap_or_default()
        .trim_start_matches(['#', ' '])
        .to_string()
}

/// True for a Markdown heading line: one or more hashes, then a space.
fn is_heading(line: &str) -> bool {
    let rest = line.trim_start_matches('#');
    rest.len() < line.len() && rest.starts_with(' ')
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use super::docs;
    use crate::collections::frontmatter::Value;
    use crate::collections::manifest::{Kind, Manifest};
    use crate::corpus::Doc;

    /// A json manifest named `cases` over `roots`, with `id` set to the entry field `name` and no other keys.
    fn manifest(roots: &[&str]) -> Manifest {
        Manifest {
            name: "cases".to_string(),
            description: None,
            owner: None,
            kind: Kind::Json,
            roots: roots.iter().map(|root| root.to_string()).collect(),
            include: Vec::new(),
            id: Some("name".to_string()),
            title: None,
            weights: BTreeMap::new(),
            marker: None,
            merge: Vec::new(),
            at: None,
        }
    }

    /// A temporary folder holding each `(path, text)` pair, with its folders made.
    fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("a temporary folder");
        for (rel, text) in files {
            let path = root.path().join(rel);
            fs::create_dir_all(path.parent().expect("a folder")).expect("a folder");
            fs::write(path, text).expect("a file");
        }
        root
    }

    /// The labels of the docs, in order.
    fn labels(found: &[Doc]) -> Vec<&str> {
        found.iter().map(|doc| doc.label.as_str()).collect()
    }

    /// MANK-JSON-01 and MANK-JSON-05: each object of the array at `at` is one doc labelled by its id, and a `title` field
    /// gives its one-liner.
    #[test]
    fn each_object_of_the_array_at_the_pointer_is_one_doc() {
        let root = tree(&[(
            "data/cases.json",
            r#"{"cases":[{"name":"NAME-001","claim":"Stable."},{"name":"NAME-002","claim":"Shared."}]}"#,
        )]);
        let mut found = manifest(&["data"]);
        found.include = vec!["*.json".to_string()];
        found.at = Some("/cases".to_string());
        found.title = Some("claim".to_string());
        let found_docs = docs(&found, root.path());
        assert_eq!(labels(&found_docs), ["cases:NAME-001", "cases:NAME-002"]);
        assert_eq!(found_docs[0].title, "Stable.");
    }

    /// MANK-JSON-02: with no `at`, the whole document is the array.
    #[test]
    fn at_defaults_to_the_document_root() {
        let root = tree(&[("data/items.json", r#"[{"name":"A"},{"name":"B"}]"#)]);
        let mut found = manifest(&["data"]);
        found.include = vec!["*.json".to_string()];
        assert_eq!(labels(&docs(&found, root.path())), ["cases:A", "cases:B"]);
    }

    /// MANK-JSON-03: an entry's scalars are its fields, and its string scalars are its text; nested values are not fields.
    #[test]
    fn scalars_are_fields_and_strings_are_the_text() {
        let root = tree(&[(
            "data/one.json",
            r#"[{"name":"X","n":2,"ok":false,"nested":{"a":"b"},"tags":["t"],"none":null,"note":"Words."}]"#,
        )]);
        let mut found = manifest(&["data"]);
        found.include = vec!["one.json".to_string()];
        let found_docs = docs(&found, root.path());
        assert_eq!(found_docs[0].text, "X\nWords.");
        let fields: Vec<(&str, Value)> = found_docs[0]
            .fields
            .iter()
            .map(|(name, value)| (name.as_str(), value.clone()))
            .collect();
        assert_eq!(
            fields,
            [
                ("n", Value::Str("2".to_string())),
                ("name", Value::Str("X".to_string())),
                ("note", Value::Str("Words.".to_string())),
                ("ok", Value::Str("false".to_string())),
            ]
        );
    }

    /// MANK-JSON-04: an entry without a text `id` field gives no doc, since the id is required.
    #[test]
    fn an_entry_without_a_text_id_gives_no_doc() {
        let root = tree(&[(
            "data/ids.json",
            r#"[{"claim":"no name"},{"name":""},"bare",{"name":"OK"}]"#,
        )]);
        let mut found = manifest(&["data"]);
        found.include = vec!["ids.json".to_string()];
        assert_eq!(labels(&docs(&found, root.path())), ["cases:OK"]);
    }

    /// MANK-JSON-06: only the files that `include` matches give docs, at any depth with `**/`.
    #[test]
    fn include_globs_choose_the_files() {
        let root = tree(&[
            ("data/a.json", r#"[{"name":"A"}]"#),
            ("data/sub/b.json", r#"[{"name":"B"}]"#),
            ("data/notes.txt", r#"[{"name":"NOT-MATCHED"}]"#),
        ]);
        let mut found = manifest(&["data"]);
        found.include = vec!["**/*.json".to_string()];
        assert_eq!(labels(&docs(&found, root.path())), ["cases:A", "cases:B"]);
        found.include = Vec::new();
        assert!(docs(&found, root.path()).is_empty());
    }

    /// MANK-JSON-07: a file that is not JSON, or whose `at` does not reach an array, gives no docs, and the others still do.
    #[test]
    fn a_file_without_an_array_at_at_gives_no_docs_and_the_others_do() {
        let root = tree(&[
            ("data/bad.json", "{not json"),
            ("data/object.json", r#"{"cases":{"name":"X"}}"#),
            ("data/good.json", r#"{"cases":[{"name":"G"}]}"#),
        ]);
        let mut found = manifest(&["data"]);
        found.include = vec!["*.json".to_string()];
        found.at = Some("/cases".to_string());
        assert_eq!(labels(&docs(&found, root.path())), ["cases:G"]);
    }

    /// MANK-JSON-08: a root may sit in a hidden folder, as for the folder kind.
    #[test]
    fn a_root_in_a_hidden_folder_is_walked() {
        let root = tree(&[(".agents/data/x.json", r#"[{"name":"HID"}]"#)]);
        let mut found = manifest(&[".agents/data"]);
        found.include = vec!["*.json".to_string()];
        assert_eq!(labels(&docs(&found, root.path())), ["cases:HID"]);
    }
}
