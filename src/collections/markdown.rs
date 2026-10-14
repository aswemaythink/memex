//! Reads a markdown manifest's docs (design §4.2): frontmatter becomes fields, the body the text.
//! Cases: MAN-MD-01, MAN-MD-02, MAN-MD-03, MAN-MD-04, MAN-MD-05, MAN-MD-06, MAN-MD-07, MAN-MD-08, MAN-MD-09, MAN-MD-10, MAN-MD-11, MAN-MD-12, MAN-MD-14, MAN-MD-15

use std::path::Path;

use serde_json::Value as Json;

use super::frontmatter::{split, Value};
use super::globs::{self, Matched};
use super::json::scalar_fields;
use super::manifest::{self, Manifest};
use super::title;
use crate::corpus::Doc;
use crate::text;

/// The globs of a markdown manifest that names none (design §4.2 `include`).
const DEFAULT_INCLUDE: &str = "**/*.md";

/// The docs of a markdown manifest, in the order of their paths. A matched file that cannot be read or parsed adds its
/// warning to `warnings` and gives no doc.
pub fn docs(manifest: &Manifest, root: &Path, warnings: &mut Vec<String>) -> Vec<Doc> {
    let default = [DEFAULT_INCLUDE.to_string()];
    let globs: &[String] = if manifest.include.is_empty() {
        &default
    } else {
        &manifest.include
    };
    let mut docs = Vec::new();
    for file in globs::matched(manifest, root, globs) {
        match read_doc(manifest, &file) {
            Ok(doc) => docs.push(doc),
            Err(line) => warnings.push(line),
        }
    }
    docs
}

/// One matched file as a doc: a `.json` file by its top-level scalars, and any other file by its frontmatter and body. A file
/// that cannot be read or used gives its warning line.
fn read_doc(manifest: &Manifest, file: &Matched) -> Result<Doc, String> {
    let text = text::read(&file.path)
        .map_err(|failure| manifest::unreadable_warning(&file.path, &failure))?;
    let (fields, body) = if file.path.extension().is_some_and(|ext| ext == "json") {
        let fields =
            sidecar_fields(&text).map_err(|reason| manifest::warning(&file.path, &reason))?;
        (fields, String::new())
    } else {
        let document = split(&text);
        (document.fields, document.body)
    };
    Ok(doc_of(manifest, file, fields, body))
}

/// The top-level scalars of a JSON object as fields, in key order. Arrays, objects and nulls are left out.
fn sidecar_fields(text: &str) -> Result<Vec<(String, Value)>, String> {
    let json: Json =
        serde_json::from_str(text).map_err(|failure| format!("not valid JSON: {failure}"))?;
    Ok(json.as_object().map_or_else(Vec::new, scalar_fields))
}

/// The doc of a matched file. Its label joins the manifest's name and the id; its path is its root-relative path.
fn doc_of(manifest: &Manifest, file: &Matched, fields: Vec<(String, Value)>, body: String) -> Doc {
    let id = id_of(manifest, file, &fields);
    Doc {
        label: format!("{}:{id}", manifest.name),
        collection: manifest.name.clone(),
        title: title::one_liner(manifest.title.as_deref(), &fields, &body),
        id,
        path: Some(file.rel.clone()),
        text: body,
        fields,
    }
}

/// The id of a doc (design §4.2 `id`): its file stem by default, its root-relative path for `path`, or the text of the
/// frontmatter field that the manifest names. A doc without that field keeps its stem.
fn id_of(manifest: &Manifest, file: &Matched, fields: &[(String, Value)]) -> String {
    match manifest.id.as_deref() {
        None | Some("stem") => stem_of(&file.rel),
        Some("path") => file.rel.clone(),
        Some(key) => text_field(fields, key).unwrap_or_else(|| stem_of(&file.rel)),
    }
}

/// The file name without its last extension.
fn stem_of(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

/// The text of a frontmatter field whose value is a scalar with some text.
fn text_field(fields: &[(String, Value)], key: &str) -> Option<String> {
    fields.iter().find_map(|(name, value)| match value {
        Value::Str(text) if name == key && !text.is_empty() => Some(text.clone()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn an_unreadable_doc_warns_with_the_cannot_read_form_and_the_others_load() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use super::docs;
        use crate::collections::manifest::Kind;
        use crate::collections::testing;

        let dir = testing::tree(&[
            ("notes/open.md", "# Open\n"),
            ("notes/locked.md", "# Locked\n"),
        ]);
        let locked = dir.path().join("notes/locked.md");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read(&locked).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let mut manifest = testing::manifest(Kind::Markdown, &["notes"]);
        manifest.name = "notes".to_string();
        let mut warnings = Vec::new();
        let found = docs(&manifest, dir.path(), &mut warnings);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).expect("restored");
        assert_eq!(testing::labels(&found), ["notes:open"]);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].starts_with("memex: warning: cannot read ")
                && warnings[0].ends_with("notes/locked.md (PermissionDenied)"),
            "{warnings:?}"
        );
    }
}
