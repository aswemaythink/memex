//! Loads and validates the manifests of design §4.2.
//! Each manifest's docs come from the reader of its kind.
//! Cases: MAN-LOAD-01, MAN-LOAD-02, MAN-LOAD-03, MAN-LOAD-04, MAN-LOAD-05, MAN-LOAD-06, MAN-LOAD-07, MAN-LOAD-08, MAN-LOAD-09, MAN-LOAD-10, MAN-LOAD-11, MAN-LOAD-12

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ignore::overrides::OverrideBuilder;
use serde_json::{Map, Value as Json};

use super::{folder, json, markdown};
use crate::corpus::Doc;
use crate::root::{lists, slash};
use crate::text;
use crate::walk::rel_string;

/// The names that belong to the built-in collections, which no manifest may take (design §3.1).
const RESERVED: [&str; 4] = ["readmes", "headers", "cases", "logs"];

/// How a manifest's roots are read (design §4.2 `kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Each matched Markdown file is one doc.
    Markdown,
    /// Each folder that holds the marker file is one doc.
    Folder,
    /// Each entry of the array at `at` is one doc.
    Json,
}

/// A collection manifest with every key of design §4.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The label prefix, which must equal the file stem.
    pub name: String,
    /// Shown by `info`.
    pub description: Option<String>,
    /// The skill that builds the collection.
    pub owner: Option<String>,
    /// How the roots are read.
    pub kind: Kind,
    /// The root-relative folders the collection reads, with `/` separators; empty for the root itself.
    pub roots: Vec<String>,
    /// Globs under the roots; markdown and json only.
    pub include: Vec<String>,
    /// The field or rule that gives each doc its id.
    pub id: Option<String>,
    /// The field that gives each doc its one-liner.
    pub title: Option<String>,
    /// The ranking boost of each field.
    pub weights: BTreeMap<String, u32>,
    /// The file whose presence defines a doc; folder only.
    pub marker: Option<String>,
    /// The globs appended to a folder doc's body; folder only.
    pub merge: Vec<String>,
    /// The JSON Pointer to the entries array; json only.
    pub at: Option<String>,
}

/// A manifest that loaded, with the number of docs its reader gave and the warnings of that read, as the corpus keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    /// The manifest itself.
    pub manifest: Manifest,
    /// The docs its reader gave.
    pub count: usize,
    /// The warnings of the files its reader could not use.
    pub warnings: Vec<String>,
}

/// The manifests under `<root>/.memex/collections/` by file name; a file that cannot be used adds a warning and is skipped.
pub fn load(root: &Path, warnings: &mut Vec<String>) -> Vec<Manifest> {
    let dir = root.join(".memex").join("collections");
    let files = listing(root, &dir).unwrap_or_else(|reason| {
        warnings.push(warning(&dir, &reason));
        Vec::new()
    });
    let mut manifests = Vec::new();
    for (file, path) in files {
        match read_one(&path, &file) {
            Ok(manifest) => manifests.push(manifest),
            Err(reason) => warnings.push(warning(&path, &reason)),
        }
    }
    manifests
}

/// The docs of one manifest, read by the reader of its kind. A file that cannot be used adds its warning to `warnings`.
pub fn docs(manifest: &Manifest, root: &Path, warnings: &mut Vec<String>) -> Vec<Doc> {
    match manifest.kind {
        Kind::Markdown => markdown::docs(manifest, root, warnings),
        Kind::Folder => folder::docs(manifest, root, warnings),
        Kind::Json => json::docs(manifest, root, warnings),
    }
}

/// The warning line for a file that cannot be used: `memex: warning: <path>: <reason>` (design §4.2).
pub fn warning(path: &Path, reason: &str) -> String {
    format!("memex: warning: {}: {reason}", slash(path))
}

/// The `.json` files of the collections folder `dir` by name: dot-named, non-UTF-8, symlink and folder entries are skipped. The
/// names `.memex` and `collections` are matched against the listings of the root and of `.memex/`, never probed on a guessed
/// path, so `.memex/Collections/` is not the folder on a case-insensitive file system (design §4.4 Name matching).
fn listing(root: &Path, dir: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let listed = lists(root, ".memex").and_then(|found| {
        Ok(found && lists(&root.join(".memex"), "collections")?)
    });
    if !listed.map_err(|failure| unreadable(&failure))? {
        return Ok(Vec::new());
    }
    let reader = match fs::read_dir(dir) {
        Ok(reader) => reader,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(failure) => return Err(unreadable(&failure)),
    };
    let mut files = Vec::new();
    for item in reader {
        let item = item.map_err(|failure| unreadable(&failure))?;
        let file_name = item.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let is_file = item
            .file_type()
            .map_err(|failure| unreadable(&failure))?
            .is_file();
        if is_file && !name.starts_with('.') && name.ends_with(".json") {
            files.push((name.to_string(), item.path()));
        }
    }
    files.sort();
    Ok(files)
}

/// Reads and validates one manifest. Its name is checked against the file name without `.json`.
fn read_one(path: &Path, file: &str) -> Result<Manifest, String> {
    let text = text::read(path).map_err(|failure| unreadable(&failure))?;
    let value: Json =
        serde_json::from_str(&text).map_err(|failure| format!("not valid JSON: {failure}"))?;
    parse(file.strip_suffix(".json").unwrap_or(file), &value)
}

/// Checks a manifest against design §4.2: its keys, the kind they belong to, and the value each takes.
fn parse(stem: &str, value: &Json) -> Result<Manifest, String> {
    let Json::Object(object) = value else {
        return Err("a manifest must be a JSON object".to_string());
    };
    if let Some(key) = object.keys().find(|key| kinds_of(key).is_none()) {
        return Err(format!("unknown key '{key}'"));
    }
    let name = name_of(object, stem)?;
    let kind = kind_of(object)?;
    if let Some(key) = object.keys().find(|key| !applies(key, kind)) {
        let word = format!("{kind:?}").to_lowercase();
        return Err(format!("key '{key}' does not apply to kind '{word}'"));
    }
    let include = strings_of(object, "include")?;
    check_globs(&include)?;
    let manifest = Manifest {
        name,
        description: text_of(object, "description")?,
        owner: text_of(object, "owner")?,
        kind,
        roots: roots_of(object)?,
        include,
        id: text_of(object, "id")?,
        title: text_of(object, "title")?,
        weights: weights_of(object)?,
        marker: text_of(object, "marker")?,
        merge: strings_of(object, "merge")?,
        at: text_of(object, "at")?,
    };
    if kind == Kind::Json && manifest.id.is_none() {
        return Err("kind 'json' needs key 'id'".to_string());
    }
    if kind == Kind::Json && manifest.include.is_empty() {
        return Err("kind 'json' needs key 'include'".to_string());
    }
    Ok(manifest)
}

/// The kinds that a manifest key belongs to (design §4.2 table). None for a key the table does not have.
fn kinds_of(key: &str) -> Option<&'static [Kind]> {
    const ALL: &[Kind] = &[Kind::Markdown, Kind::Folder, Kind::Json];
    match key {
        "name" | "description" | "owner" | "kind" | "roots" | "id" | "title" | "weights" => {
            Some(ALL)
        }
        "include" => Some(&[Kind::Markdown, Kind::Json]),
        "marker" | "merge" => Some(&[Kind::Folder]),
        "at" => Some(&[Kind::Json]),
        _ => None,
    }
}

/// True when a key belongs to the kind. A key the table does not have belongs to no kind.
fn applies(key: &str, kind: Kind) -> bool {
    kinds_of(key).is_some_and(|kinds| kinds.contains(&kind))
}

/// The collection name: present, equal to the file stem, and not a built-in name.
fn name_of(object: &Map<String, Json>, stem: &str) -> Result<String, String> {
    let name = text_of(object, "name")?.ok_or("missing key 'name'")?;
    if name != stem {
        return Err(format!("name '{name}' must match the file name '{stem}'"));
    }
    if RESERVED.contains(&name.as_str()) {
        return Err(format!("name '{name}' is reserved"));
    }
    Ok(name)
}

/// The kind: present, and one of the three words of design §4.2.
fn kind_of(object: &Map<String, Json>) -> Result<Kind, String> {
    let word = text_of(object, "kind")?.ok_or("missing key 'kind'")?;
    match word.as_str() {
        "markdown" => Ok(Kind::Markdown),
        "folder" => Ok(Kind::Folder),
        "json" => Ok(Kind::Json),
        other => Err(format!("kind '{other}' is not markdown, folder or json")),
    }
}

/// The roots: present, and each a folder inside the root, written relative to it.
fn roots_of(object: &Map<String, Json>) -> Result<Vec<String>, String> {
    match object.get("roots") {
        Some(Json::Array(items)) => items.iter().map(root_of).collect(),
        Some(_) => Err("key 'roots' must be a list of folders".to_string()),
        None => Err("missing key 'roots'".to_string()),
    }
}

/// One root, normalised to `/` separators; an absolute root, or one that climbs with `..`, is refused.
fn root_of(item: &Json) -> Result<String, String> {
    let text = item
        .as_str()
        .ok_or("key 'roots' must be a list of folders")?;
    rel_string(Path::new(text))
        .ok_or_else(|| format!("root '{text}' is not a folder inside the root"))
}

/// The text of a key whose value is a string, or None when the key is absent.
fn text_of(object: &Map<String, Json>, key: &str) -> Result<Option<String>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(Json::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(format!("key '{key}' must be a string")),
    }
}

/// The strings of a key whose value is a list, or none when the key is absent.
fn strings_of(object: &Map<String, Json>, key: &str) -> Result<Vec<String>, String> {
    let Some(value) = object.get(key) else {
        return Ok(Vec::new());
    };
    let Json::Array(items) = value else {
        return Err(format!("key '{key}' must be a list of strings"));
    };
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("key '{key}' must be a list of strings"))
        })
        .collect()
}

/// The weights: an object whose values are whole numbers, or none when the key is absent.
fn weights_of(object: &Map<String, Json>) -> Result<BTreeMap<String, u32>, String> {
    let Some(value) = object.get("weights") else {
        return Ok(BTreeMap::new());
    };
    let Json::Object(fields) = value else {
        return Err("key 'weights' must be an object".to_string());
    };
    fields
        .iter()
        .map(|(field, weight)| {
            weight
                .as_u64()
                .and_then(|number| u32::try_from(number).ok())
                .map(|number| (field.clone(), number))
                .ok_or_else(|| format!("weight '{field}' must be a whole number"))
        })
        .collect()
}

/// Refuses a glob that the override matcher cannot parse, so that a bad `include` is a warning at load.
fn check_globs(globs: &[String]) -> Result<(), String> {
    let bad = globs
        .iter()
        .find(|glob| OverrideBuilder::new("").add(glob).is_err());
    bad.map_or(Ok(()), |glob| {
        Err(format!("include '{glob}' is not a valid glob"))
    })
}

/// The reason for a file that cannot be read: the io kind, never the OS text (design §4.4).
pub fn unreadable(failure: &io::Error) -> String {
    format!("cannot read it ({:?})", failure.kind())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_collections_folder_in_another_case_is_not_the_folder() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let other = dir.path().join(".memex").join("Collections");
        fs::create_dir_all(&other).expect("a folder");
        let manifest = r#"{"name": "notes", "kind": "markdown", "roots": ["."]}"#;
        fs::write(other.join("notes.json"), manifest).expect("a manifest");
        let mut warnings = Vec::new();
        assert!(load(dir.path(), &mut warnings).is_empty());
        assert!(warnings.is_empty(), "{warnings:?}");
        let exact = dir.path().join(".memex").join("collections");
        fs::rename(&other, &exact).expect("renamed");
        assert_eq!(load(dir.path(), &mut warnings).len(), 1, "{warnings:?}");
    }
}
