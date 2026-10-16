//! Reads a `folder` manifest's docs: each folder with its marker file is one doc (design §4.2).
//! The marker gives the fields and, when it is Markdown, the start of the text; `merge` files add the rest, and one that cannot be read warns.
//! Cases: MANK-FOLDER-01, MANK-FOLDER-02, MANK-FOLDER-03, MANK-FOLDER-04, MANK-FOLDER-05, MANK-FOLDER-06, MANK-FOLDER-07, MANK-FOLDER-08, MANK-FOLDER-09, MANK-MERGE-01, MANK-CITE-01

use std::collections::BTreeMap;
use std::path::Path;

use super::frontmatter::{self, Value};
use super::globs;
use super::json::{doc, field_text, folder_under, read_json, scalar_fields, warn};
use super::manifest::{self, Manifest};
use crate::corpus::{name_of, parent_of, Doc};
use crate::intent::header;
use crate::text;
use crate::walk::{self, Entry};

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
            let rel_folder = parent_of(&file.rel).unwrap_or_default();
            let siblings = files
                .get(rel_folder)
                .map_or(&[][..], |list| list.as_slice());
            let folder = folder_under(dir, rel_folder);
            docs.extend(folder_doc(
                manifest, &base, file, siblings, warnings, folder,
            ));
        }
    }
    docs
}

/// The doc of the folder that holds `marker`, whose relative citations resolve from `folder`, the folder's root-relative path (design
/// §2.3). A marker that cannot be read or parsed, or whose id is missing or empty, adds its warning to `warnings` and gives no doc.
fn folder_doc(
    manifest: &Manifest,
    base: &Path,
    marker: &Entry,
    siblings: &[&Entry],
    warnings: &mut Vec<String>,
    folder: String,
) -> Option<Doc> {
    let parsed = marker_fields(&marker.path).and_then(|(fields, body)| {
        let id = id_of(manifest, base, marker, &fields)
            .map_err(|reason| manifest::warning(&marker.path, &reason))?;
        Ok((id, fields, body))
    });
    let (id, fields, body) = match parsed {
        Ok(found) => found,
        Err(line) => {
            warn(warnings, line);
            return None;
        }
    };
    if id.is_empty() {
        return None;
    }
    let text = joined(body, merge_pieces(manifest, marker, siblings, warnings));
    Some(doc(manifest, id, text.clone(), fields, &text, folder))
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
/// and body. Any other marker gives neither. A JSON or Markdown marker that cannot be read or parsed gives its warning line.
fn marker_fields(path: &Path) -> Result<(Vec<(String, Value)>, String), String> {
    match extension(path) {
        Some("json") => {
            let document = read_json(path)?;
            let fields = document.as_object().map_or_else(Vec::new, scalar_fields);
            Ok((fields, String::new()))
        }
        Some("md") => {
            let contents =
                text::read(path).map_err(|failure| manifest::unreadable_warning(path, &failure))?;
            let document = frontmatter::split(&contents);
            Ok((document.fields, document.body))
        }
        _ => Ok((Vec::new(), String::new())),
    }
}

/// The text the `merge` globs add to a marker's folder: each matching file except the marker, in name order. A matching file that
/// cannot be read adds its warning to `warnings`, and adds nothing (design §4.2 Unreadable files).
fn merge_pieces(
    manifest: &Manifest,
    marker: &Entry,
    siblings: &[&Entry],
    warnings: &mut Vec<String>,
) -> Vec<String> {
    let folder = marker.path.parent().unwrap_or_else(|| Path::new(""));
    let others: Vec<&Entry> = siblings
        .iter()
        .copied()
        .filter(|entry| entry.rel != marker.rel)
        .collect();
    globs::hits(folder, &manifest.merge, &others)
        .into_iter()
        .filter_map(|entry| match merge_piece(&entry.path) {
            Ok(piece) => piece,
            Err(line) => {
                warn(warnings, line);
                None
            }
        })
        .collect()
}

/// What a merged file adds: a Markdown file in full, or a code file's header by the rule of design §2.1, or nothing for any other
/// file. A file that cannot be read gives its warning line.
fn merge_piece(path: &Path) -> Result<Option<String>, String> {
    if extension(path) == Some("md") {
        return text::read(path)
            .map(Some)
            .map_err(|failure| manifest::unreadable_warning(path, &failure));
    }
    match header::read_head(path) {
        Ok(head) => Ok(header::header(path, &head)),
        Err(failure) => Err(manifest::unreadable_warning(path, &failure)),
    }
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

/// The extension of a path, when it is UTF-8.
fn extension(path: &Path) -> Option<&str> {
    path.extension()?.to_str()
}
