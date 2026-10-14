//! Reads a `json` manifest's docs: each object of the array at `at` is one doc (design §4.2).
//! It also holds the helpers that the folder and markdown kinds share with it: fields, one-liners and docs.
//! Cases: MANK-JSON-01, MANK-JSON-02, MANK-JSON-03, MANK-JSON-04, MANK-JSON-05, MANK-JSON-06, MANK-JSON-07, MANK-JSON-08

use std::path::Path;

use serde_json::{Map, Value as Json};

use super::frontmatter::Value;
use super::globs;
use super::manifest::{self, Manifest};
use crate::corpus::Doc;
use crate::intent::sentence;
use crate::text;
use crate::walk::{self, Entry};

/// The docs of a json manifest: one per object, in each matched file, whose `id` field is text. A manifest with no `id` key
/// gives none, since the id is required. A file that cannot be used adds its warning to `warnings`.
pub fn docs(manifest: &Manifest, root: &Path, warnings: &mut Vec<String>) -> Vec<Doc> {
    let Some(key) = manifest.id.as_deref() else {
        return Vec::new();
    };
    let at = manifest.at.as_deref().unwrap_or("");
    let mut docs = Vec::new();
    for dir in &manifest.roots {
        let base = root.join(dir);
        let entries = walk::walk(&base);
        let files: Vec<&Entry> = entries.iter().filter(|entry| !entry.is_dir).collect();
        for file in globs::hits(&base, &manifest.include, &files) {
            for item in items_at(&file.path, at, warnings) {
                docs.extend(entry_doc(manifest, key, &item));
            }
        }
    }
    docs
}

/// The items of the array that `at` points to in the JSON file at `path`. A file that cannot be read or parsed adds its
/// warning and gives none. A file whose `at` does not reach an array has none, and no warning.
fn items_at(path: &Path, at: &str, warnings: &mut Vec<String>) -> Vec<Json> {
    let document = match read_json(path) {
        Ok(document) => document,
        Err(line) => {
            warn(warnings, line);
            return Vec::new();
        }
    };
    match document.pointer(at) {
        Some(Json::Array(items)) => items.clone(),
        _ => Vec::new(),
    }
}

/// The JSON value of the file at `path`, or the warning line for it: it cannot be read, or it is not JSON.
pub(super) fn read_json(path: &Path) -> Result<Json, String> {
    let contents =
        text::read(path).map_err(|failure| manifest::unreadable_warning(path, &failure))?;
    serde_json::from_str(&contents)
        .map_err(|failure| manifest::warning(path, &format!("not valid JSON: {failure}")))
}

/// Adds a warning line to `warnings`, once: a line that is already there is not repeated.
pub(super) fn warn(warnings: &mut Vec<String>, line: String) {
    if !warnings.contains(&line) {
        warnings.push(line);
    }
}

/// The doc of one entry: its scalars as fields, its string scalars as the text, and its `id` field as the id. None when the
/// entry is not an object, or its `id` is missing or empty. Its default one-liner is read without the id field.
fn entry_doc(manifest: &Manifest, key: &str, item: &Json) -> Option<Doc> {
    let members = item.as_object()?;
    let fields = scalar_fields(members);
    let id = field_text(&fields, key).filter(|id| !id.is_empty())?;
    let lead = text_of(members, Some(key));
    Some(doc(manifest, id, text_of(members, None), fields, &lead))
}

/// The text of an entry: its string scalars, one per line, in source order. The member named `skip`, when there is one, is
/// left out.
fn text_of(members: &Map<String, Json>, skip: Option<&str>) -> String {
    members
        .iter()
        .filter(|(name, _)| Some(name.as_str()) != skip)
        .filter_map(|(_, value)| value.as_str())
        .collect::<Vec<&str>>()
        .join("\n")
}

/// The fields that an object's top-level scalars give, in source order: strings, numbers and booleans as text. Nulls, arrays
/// and nested objects give none.
pub(super) fn scalar_fields(members: &Map<String, Json>) -> Vec<(String, Value)> {
    members
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

/// The doc of a manifest: labelled `<name>:<id>`, with its one-liner from the `title` field, else from the first sentence of
/// `lead`, the text that the default line is read from.
pub(super) fn doc(
    manifest: &Manifest,
    id: String,
    text: String,
    fields: Vec<(String, Value)>,
    lead: &str,
) -> Doc {
    let title = one_liner(manifest.title.as_deref(), &fields, lead);
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

/// The one-liner: the text of the field that the manifest's `title` names, else the default of `lead`, as its first sentence.
fn one_liner(title: Option<&str>, fields: &[(String, Value)], lead: &str) -> String {
    let line = title
        .and_then(|key| field_text(fields, key))
        .unwrap_or_else(|| default_title(lead));
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
    use super::docs;
    use crate::collections::frontmatter::Value;
    use crate::collections::manifest::{Kind, Manifest};
    use crate::collections::testing::{self, labels, tree};

    /// A json manifest named `cases` over `roots`, with `id` set to the entry field `name` and no other keys.
    fn manifest(roots: &[&str]) -> Manifest {
        let mut found = testing::manifest(Kind::Json, roots);
        found.id = Some("name".to_string());
        found
    }

    /// MANK-JSON-01, MANK-JSON-02 and MANK-JSON-05: each object of the array at the pointer is one doc labelled by its id, and
    /// a `title` field gives its one-liner.
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
        let found_docs = docs(&found, root.path(), &mut Vec::new());
        assert_eq!(labels(&found_docs), ["cases:NAME-001", "cases:NAME-002"]);
        assert_eq!(found_docs[0].title, "Stable.");
    }

    /// MANK-JSON-03: an entry's scalars are its fields in source order, and its string scalars are its text.
    #[test]
    fn scalars_are_fields_in_source_order_and_strings_are_the_text() {
        let root = tree(&[(
            "data/one.json",
            r#"[{"name":"X","n":2,"ok":false,"nested":{"a":"b"},"tags":["t"],"none":null,"note":"Words."}]"#,
        )]);
        let mut found = manifest(&["data"]);
        found.include = vec!["one.json".to_string()];
        let found_docs = docs(&found, root.path(), &mut Vec::new());
        assert_eq!(found_docs[0].text, "X\nWords.");
        let fields: Vec<(&str, Value)> = found_docs[0]
            .fields
            .iter()
            .map(|(name, value)| (name.as_str(), value.clone()))
            .collect();
        assert_eq!(
            fields,
            [
                ("name", Value::Str("X".to_string())),
                ("n", Value::Str("2".to_string())),
                ("ok", Value::Str("false".to_string())),
                ("note", Value::Str("Words.".to_string())),
            ]
        );
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
        assert_eq!(
            labels(&docs(&found, root.path(), &mut Vec::new())),
            ["cases:A", "cases:B"]
        );
        found.include = Vec::new();
        assert!(docs(&found, root.path(), &mut Vec::new()).is_empty());
    }

    /// MANK-JSON-07: a file that is not JSON gives no docs and a warning, a file whose `at` does not reach an array gives no
    /// docs and no warning, and the others still do.
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
        let mut warnings = Vec::new();
        assert_eq!(
            labels(&docs(&found, root.path(), &mut warnings)),
            ["cases:G"]
        );
        assert_eq!(warnings.len(), 1, "only the file that is not JSON warns");
    }

    /// MANK-JSON-02: with no `title` field, an entry's one-liner is read from its text without the id field.
    #[test]
    fn the_default_one_liner_skips_the_id_field() {
        let root = tree(&[(
            "data/cases.json",
            r#"[{"name":"NAME-001","claim":"Stable."}]"#,
        )]);
        let mut found = manifest(&["data"]);
        found.include = vec!["*.json".to_string()];
        assert_eq!(
            docs(&found, root.path(), &mut Vec::new())[0].title,
            "Stable."
        );
    }

    /// An unreadable JSON file warns as `cannot read <path> (<ErrorKind>)`, and the other files still give docs.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_json_file_warns_with_the_cannot_read_form() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let root = tree(&[
            ("data/locked.json", r#"[{"name":"L"}]"#),
            ("data/open.json", r#"[{"name":"O"}]"#),
        ]);
        let locked = root.path().join("data/locked.json");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read(&locked).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let mut found = manifest(&["data"]);
        found.include = vec!["*.json".to_string()];
        let mut warnings = Vec::new();
        let found_docs = docs(&found, root.path(), &mut warnings);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).expect("restored");
        assert_eq!(labels(&found_docs), ["cases:O"]);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].starts_with("memex: warning: cannot read ")
                && warnings[0].ends_with("data/locked.json (PermissionDenied)"),
            "{warnings:?}"
        );
    }
}
