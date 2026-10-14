//! Reads a markdown manifest's docs (design §4.2): frontmatter becomes fields, the body the text.
//! Cases: MAN-MD-01, MAN-MD-02, MAN-MD-03, MAN-MD-04, MAN-MD-05, MAN-MD-06, MAN-MD-07, MAN-MD-08, MAN-MD-09, MAN-MD-10, MAN-MD-11, MAN-MD-12

use std::path::Path;

use serde_json::Value as Json;

use super::frontmatter::{split, Value};
use super::manifest::{self, Manifest, Matched};
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
    for file in manifest::matched(manifest, root, globs) {
        match read_doc(manifest, &file) {
            Ok(doc) => docs.push(doc),
            Err(reason) => warnings.push(manifest::warning(&file.path, &reason)),
        }
    }
    docs
}

/// One matched file as a doc: a `.json` file by its top-level scalars, and any other file by its frontmatter and body.
fn read_doc(manifest: &Manifest, file: &Matched) -> Result<Doc, String> {
    let text = text::read(&file.path).map_err(|failure| manifest::unreadable(&failure))?;
    let (fields, body) = if file.path.extension().is_some_and(|ext| ext == "json") {
        (sidecar_fields(&text)?, String::new())
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
    let Json::Object(fields) = json else {
        return Ok(Vec::new());
    };
    Ok(fields
        .iter()
        .filter_map(|(key, value)| scalar_text(value).map(|text| (key.clone(), Value::Str(text))))
        .collect())
}

/// A scalar as the text of a field: a string as it is, and a number or a boolean as written.
fn scalar_text(value: &Json) -> Option<String> {
    match value {
        Json::String(text) => Some(text.clone()),
        Json::Number(number) => Some(number.to_string()),
        Json::Bool(flag) => Some(flag.to_string()),
        Json::Null | Json::Array(_) | Json::Object(_) => None,
    }
}

/// The doc of a matched file. Its label joins the manifest's name and the id; its path is its root-relative path.
fn doc_of(manifest: &Manifest, file: &Matched, fields: Vec<(String, Value)>, body: String) -> Doc {
    let id = id_of(manifest, file, &fields);
    Doc {
        label: format!("{}:{id}", manifest.name),
        collection: manifest.name.clone(),
        title: title_of(manifest, &fields, &body),
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

/// The one-liner of a doc (design §4.2 `title`): the text of the field that the manifest names, else the body's default.
fn title_of(manifest: &Manifest, fields: &[(String, Value)], body: &str) -> String {
    manifest
        .title
        .as_deref()
        .and_then(|key| text_field(fields, key))
        .unwrap_or_else(|| default_title(body))
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

/// The default one-liner: the first heading outside a code fence, else the first line with text, else nothing.
fn default_title(body: &str) -> String {
    heading(body)
        .or_else(|| first_line(body))
        .unwrap_or_default()
}

/// The text of the first ATX heading that is not inside a code fence.
fn heading(body: &str) -> Option<String> {
    let mut fenced = false;
    for line in body.lines() {
        let line = line.trim_start();
        if line.starts_with("```") || line.starts_with("~~~") {
            fenced = !fenced;
        } else if !fenced {
            if let Some(text) = heading_text(line) {
                return Some(text);
            }
        }
    }
    None
}

/// The text of an ATX heading line: one to six `#` marks, then a space or the end of the line, and the text after them.
fn heading_text(line: &str) -> Option<String> {
    let marks = line.bytes().take_while(|byte| *byte == b'#').count();
    let rest = &line[marks..];
    let is_heading = (1..=6).contains(&marks) && (rest.is_empty() || rest.starts_with([' ', '\t']));
    let text = rest.trim();
    (is_heading && !text.is_empty()).then(|| text.to_string())
}

/// The first line of the body that has any text, trimmed.
fn first_line(body: &str) -> Option<String> {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_skip_fences_and_need_a_space_after_the_marks() {
        assert_eq!(default_title("```\n# no\n```\n# Yes\n"), "Yes");
        assert_eq!(default_title("#tag line\nSecond\n"), "#tag line");
        assert_eq!(default_title("##  Two  \n"), "Two");
        assert_eq!(default_title("\n\n  Plain  \n"), "Plain");
        assert_eq!(default_title(""), "");
    }
}
