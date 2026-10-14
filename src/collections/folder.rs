//! Reads a `folder` manifest's docs: each folder with its marker file is one doc (design §4.2).
//! The marker gives the fields and, when it is Markdown, the start of the text; `merge` files add the rest.
//! Cases: MANK-FOLDER-01, MANK-FOLDER-02, MANK-FOLDER-03, MANK-FOLDER-04, MANK-FOLDER-05, MANK-FOLDER-06, MANK-FOLDER-07, MANK-FOLDER-08, MANK-FOLDER-09

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::frontmatter::{self, Value};
use super::json::{doc, field_text, glob_hits, read_json, scalar_fields, warn};
use super::manifest::{self, Manifest};
use crate::corpus::{parent_of, Doc};
use crate::intent::header;
use crate::text;
use crate::walk::{self, Entry};

/// The most bytes of a code file that a merge reads, for its header (design §2.1).
const HEAD_BYTES: u64 = 8 * 1024;

/// The docs of a folder manifest: one per folder under its roots that holds the marker file. A marker that cannot be read or
/// parsed, or whose `id` field is missing or empty, adds its warning to `warnings` and gives no doc.
pub fn docs(manifest: &Manifest, root: &Path, warnings: &mut Vec<String>) -> Vec<Doc> {
    let Some(marker) = manifest.marker.as_deref() else {
        return Vec::new();
    };
    let mut docs = Vec::new();
    for dir in &manifest.roots {
        let base = root.join(dir);
        let entries = walk::walk(&base);
        let files = files_by_folder(&entries);
        for file in entries.iter().filter(|entry| is_marker(entry, marker)) {
            let folder = parent_of(&file.rel).unwrap_or_default();
            let siblings = files.get(folder).map_or(&[][..], |list| list.as_slice());
            docs.extend(folder_doc(manifest, &base, file, siblings, warnings));
        }
    }
    docs
}

/// The doc of the folder that holds `marker`. A marker that cannot be read or parsed, or whose id is missing or empty, adds its
/// warning to `warnings` and gives no doc.
fn folder_doc(
    manifest: &Manifest,
    base: &Path,
    marker: &Entry,
    siblings: &[&Entry],
    warnings: &mut Vec<String>,
) -> Option<Doc> {
    let parsed = marker_fields(&marker.path).and_then(|(fields, body)| {
        let id = id_of(manifest, base, marker, &fields)?;
        Ok((id, fields, body))
    });
    let (id, fields, body) = match parsed {
        Ok(found) => found,
        Err(reason) => {
            warn(warnings, &marker.path, &reason);
            return None;
        }
    };
    if id.is_empty() {
        return None;
    }
    let text = joined(body, merge_pieces(manifest, marker, siblings));
    Some(doc(manifest, id, text.clone(), fields, &text))
}

/// The id of a folder doc: the marker field that the manifest's `id` names, which must be present and not empty, or else the
/// folder's own name.
fn id_of(
    manifest: &Manifest,
    base: &Path,
    marker: &Entry,
    fields: &[(String, Value)],
) -> Result<String, String> {
    match manifest.id.as_deref() {
        Some(key) => field_text(fields, key)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| format!("no '{key}' field")),
        None => Ok(folder_name(
            base,
            parent_of(&marker.rel).unwrap_or_default(),
        )),
    }
}

/// The fields and body that a marker gives: a JSON marker's top-level scalars and no body, or a Markdown marker's frontmatter
/// and body. Any other marker gives neither. A JSON or Markdown marker that cannot be read or parsed gives its reason.
fn marker_fields(path: &Path) -> Result<(Vec<(String, Value)>, String), String> {
    match extension(path) {
        Some("json") => {
            let document = read_json(path)?;
            let fields = document.as_object().map_or_else(Vec::new, scalar_fields);
            Ok((fields, String::new()))
        }
        Some("md") => {
            let contents = text::read(path).map_err(|failure| manifest::unreadable(&failure))?;
            let document = frontmatter::split(&contents);
            Ok((document.fields, document.body))
        }
        _ => Ok((Vec::new(), String::new())),
    }
}

/// The text the `merge` globs add to a marker's folder: each matching file except the marker, in name order.
fn merge_pieces(manifest: &Manifest, marker: &Entry, siblings: &[&Entry]) -> Vec<String> {
    let folder = marker.path.parent().unwrap_or_else(|| Path::new(""));
    let others: Vec<&Entry> = siblings
        .iter()
        .copied()
        .filter(|entry| entry.rel != marker.rel)
        .collect();
    glob_hits(folder, &manifest.merge, &others)
        .into_iter()
        .filter_map(|entry| merge_piece(&entry.path))
        .collect()
}

/// What a merged file adds: a Markdown file in full, or a code file's header by the rule of design §2.1. Other files add nothing.
fn merge_piece(path: &Path) -> Option<String> {
    if extension(path) == Some("md") {
        return read_text(path);
    }
    header::header(path, &read_head(path)?)
}

/// The files of a walk, grouped by the root-relative folder that holds them, in walk order.
fn files_by_folder(entries: &[Entry]) -> BTreeMap<&str, Vec<&Entry>> {
    let mut index: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    for entry in entries.iter().filter(|entry| !entry.is_dir) {
        index
            .entry(parent_of(&entry.rel).unwrap_or_default())
            .or_default()
            .push(entry);
    }
    index
}

/// True for a file whose name is exactly the marker's name.
fn is_marker(entry: &Entry, marker: &str) -> bool {
    !entry.is_dir && name_of(&entry.rel) == marker
}

/// The text of a doc: its body, then each piece, separated by blank lines. Blank pieces are dropped.
fn joined(body: String, pieces: Vec<String>) -> String {
    std::iter::once(body)
        .chain(pieces)
        .map(|piece| piece.trim_matches(['\r', '\n']).trim_end().to_string())
        .filter(|piece| !piece.is_empty())
        .collect::<Vec<String>>()
        .join("\n\n")
}

/// The default id of a folder doc: the folder's own name, which for the root of a walk is the root's name.
fn folder_name(base: &Path, folder: &str) -> String {
    if folder.is_empty() {
        base.file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
    } else {
        name_of(folder).to_string()
    }
}

/// The last component of a root-relative path.
fn name_of(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

/// The extension of a path, when it is UTF-8.
fn extension(path: &Path) -> Option<&str> {
    path.extension()?.to_str()
}

/// The whole text of a file, read as the shared helper reads it, or None when it cannot be read.
fn read_text(path: &Path) -> Option<String> {
    text::read(path).ok()
}

/// The first `HEAD_BYTES` of a file, or None when it cannot be read.
fn read_head(path: &Path) -> Option<Vec<u8>> {
    let mut head = Vec::new();
    File::open(path)
        .ok()?
        .take(HEAD_BYTES)
        .read_to_end(&mut head)
        .ok()?;
    Some(head)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use super::docs;
    use crate::collections::frontmatter::Value;
    use crate::collections::manifest::{Kind, Manifest};

    /// A folder manifest named `cases` over `roots`, with the marker `case.json` and no other keys.
    fn manifest(roots: &[&str]) -> Manifest {
        Manifest {
            name: "cases".to_string(),
            description: None,
            owner: None,
            kind: Kind::Folder,
            roots: roots.iter().map(|root| root.to_string()).collect(),
            include: Vec::new(),
            id: None,
            title: None,
            weights: BTreeMap::new(),
            marker: Some("case.json".to_string()),
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
    fn labels(found: &[crate::corpus::Doc]) -> Vec<&str> {
        found.iter().map(|doc| doc.label.as_str()).collect()
    }

    /// MANK-FOLDER-01 and MANK-FOLDER-02: each marker folder is one doc, labelled by its id, with the top-level scalars as its
    /// fields; a folder with no marker gives none.
    #[test]
    fn marker_folders_are_docs_with_their_top_level_scalars() {
        let root = tree(&[
            (
                "cases/a/case.json",
                r#"{"id":"NAME-001","name":"Stable names","count":3,"ok":true,"nested":{"x":"y"},"list":["z"],"gone":null}"#,
            ),
            ("cases/b/notes.txt", "no marker here"),
            ("cases/c/case.json", r#"{"id":"NAME-002"}"#),
        ]);
        let mut found = manifest(&["cases"]);
        found.id = Some("id".to_string());
        found.title = Some("name".to_string());
        let found_docs = docs(&found, root.path(), &mut Vec::new());
        assert_eq!(labels(&found_docs), ["cases:NAME-001", "cases:NAME-002"]);
        let names: Vec<&str> = found_docs[0]
            .fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["id", "name", "count", "ok"]);
        assert_eq!(found_docs[0].fields[2].1, Value::Str("3".to_string()));
        assert_eq!(found_docs[0].title, "Stable names");
        assert_eq!(found_docs[0].path, None);
    }

    /// MANK-FOLDER-03: a Markdown marker's frontmatter gives the fields, and its body starts the text.
    #[test]
    fn a_markdown_marker_gives_its_frontmatter_and_its_body() {
        let root = tree(&[(
            "cases/NOTE-1/case.md",
            "---\nid: NOTE-1\ntags: [a, b]\n---\n# Heading\n\nBody text.\n",
        )]);
        let mut found = manifest(&["cases"]);
        found.marker = Some("case.md".to_string());
        found.id = Some("id".to_string());
        let found_docs = docs(&found, root.path(), &mut Vec::new());
        assert_eq!(labels(&found_docs), ["cases:NOTE-1"]);
        assert_eq!(
            found_docs[0].fields,
            vec![
                ("id".to_string(), Value::Str("NOTE-1".to_string())),
                (
                    "tags".to_string(),
                    Value::List(vec!["a".to_string(), "b".to_string()])
                ),
            ]
        );
        assert_eq!(found_docs[0].text, "# Heading\n\nBody text.");
        assert_eq!(found_docs[0].title, "Heading");
    }

    /// MANK-FOLDER-04 and MANK-FOLDER-05: a merged Markdown file adds its full text, and a merged code file adds only its
    /// header; other files are not merged.
    #[test]
    fn merged_markdown_is_whole_and_merged_code_is_its_header() {
        let root = tree(&[
            ("cases/a/case.json", r#"{"id":"NAME-001"}"#),
            ("cases/a/README.md", "# Case one\n\nThe claim.\n"),
            (
                "cases/a/spec.rs",
                "//! Checks the claim.\n//! Cases: NAME-001\n\nfn secret_body() {}\n",
            ),
            ("cases/a/other.txt", "not merged"),
        ]);
        let mut found = manifest(&["cases"]);
        found.id = Some("id".to_string());
        found.merge = vec!["README.md".to_string(), "spec.rs".to_string()];
        let found_docs = docs(&found, root.path(), &mut Vec::new());
        assert_eq!(
            found_docs[0].text,
            "# Case one\n\nThe claim.\n\nChecks the claim.\nCases: NAME-001"
        );
        assert_eq!(found_docs[0].title, "Case one");
    }

    /// MANK-FOLDER-06 and MANK-FOLDER-07: with no `id` key the folder's name is the id, and with no `title` key the one-liner
    /// is the first sentence of the first line.
    #[test]
    fn with_no_keys_the_folder_names_the_doc_and_its_first_sentence_titles_it() {
        let root = tree(&[
            ("cases/NAME-009/case.json", "{}"),
            ("cases/NAME-009/README.md", "Plain first line. More.\n"),
        ]);
        let mut found = manifest(&["cases"]);
        found.merge = vec!["README.md".to_string()];
        let found_docs = docs(&found, root.path(), &mut Vec::new());
        assert_eq!(labels(&found_docs), ["cases:NAME-009"]);
        assert_eq!(found_docs[0].title, "Plain first line.");
    }

    /// MANK-FOLDER-06: with `id` naming a marker field, a marker without that field gives no doc.
    #[test]
    fn a_marker_without_the_named_id_field_gives_no_doc() {
        let root = tree(&[("cases/x/case.json", r#"{"name":"no id here"}"#)]);
        let mut found = manifest(&["cases"]);
        found.id = Some("id".to_string());
        assert!(docs(&found, root.path(), &mut Vec::new()).is_empty());
    }

    /// MANK-FOLDER-08: a root may sit in a hidden folder, and the hidden entries below it are skipped.
    #[test]
    fn a_root_in_a_hidden_folder_is_walked_and_its_hidden_children_are_not() {
        let root = tree(&[
            (".agents/cases/a/case.json", r#"{"id":"HID-01"}"#),
            (".agents/cases/.hidden/case.json", r#"{"id":"HID-02"}"#),
        ]);
        let mut found = manifest(&[".agents/cases"]);
        found.id = Some("id".to_string());
        assert_eq!(
            labels(&docs(&found, root.path(), &mut Vec::new())),
            ["cases:HID-01"]
        );
    }

    /// MANK-FOLDER-09: a marker that is not JSON gives no doc, and the other folders still give theirs.
    #[test]
    fn a_marker_that_is_not_json_gives_no_doc_and_the_others_do() {
        let root = tree(&[
            ("cases/bad/case.json", "{not json"),
            ("cases/good/case.json", r#"{"id":"GOOD-1"}"#),
        ]);
        let mut found = manifest(&["cases"]);
        found.id = Some("id".to_string());
        assert_eq!(
            labels(&docs(&found, root.path(), &mut Vec::new())),
            ["cases:GOOD-1"]
        );
    }
}
