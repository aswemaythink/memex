//! Reads the log store's entries as docs for `show` (design §3.1 and §4.3).
//! Every non-dot `.md` file one folder below the store is one doc, labelled `stream:id`, with its frontmatter as fields and its title from its first line. A stream folder or an entry that cannot be read is a warning, and is skipped (design §4.2).
//! Cases: SHOW-LOG-01, SHOW-LOG-02, SHOW-JSON-03

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::collections::frontmatter::split;
use crate::corpus::Doc;
use crate::error::Error;
use crate::logs::write::title_of;
use crate::root::{slash, store_listed};
use crate::text;

/// Every entry in the store, as docs. A store that is not listed under its parent by its exact name, or is missing, has none.
/// A stream folder or an entry that cannot be read adds its warning to `warnings` and is skipped (design §4.2), and so does a
/// store whose listing cannot be read.
pub fn load(store: &Path, warnings: &mut Vec<String>) -> Vec<Doc> {
    match store_listed(store) {
        Ok(true) => {}
        Ok(false) => return Vec::new(),
        Err(failure) => {
            warnings.push(warning(store, &failure));
            return Vec::new();
        }
    }
    let mut docs = Vec::new();
    for (stream, folder) in entries(store, true, warnings) {
        for (file, path) in entries(&folder, false, warnings) {
            match entry_doc(&stream, &file, &path) {
                Ok(doc) => docs.push(doc),
                Err(error) => warnings.push(format!("memex: warning: {}", error.message)),
            }
        }
    }
    docs
}

/// The non-dot entries of `dir` by name, sorted: its folders when `folders` is set, otherwise its `.md` files. A missing
/// `dir` has none. A `dir` that cannot be listed, or an entry whose type cannot be read, adds its warning and is skipped.
fn entries(dir: &Path, folders: bool, warnings: &mut Vec<String>) -> Vec<(String, PathBuf)> {
    let reader = match fs::read_dir(dir) {
        Ok(reader) => reader,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Vec::new(),
        Err(failure) => {
            warnings.push(warning(dir, &failure));
            return Vec::new();
        }
    };
    let mut found = Vec::new();
    for item in reader {
        let item = match item {
            Ok(item) => item,
            Err(failure) => {
                warnings.push(warning(dir, &failure));
                continue;
            }
        };
        let Some(name) = item.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let kind = match item.file_type() {
            Ok(kind) => kind,
            Err(failure) => {
                warnings.push(warning(&item.path(), &failure));
                continue;
            }
        };
        let wanted = if folders {
            kind.is_dir()
        } else {
            kind.is_file() && name.ends_with(".md")
        };
        if wanted && !name.starts_with('.') {
            found.push((name, item.path()));
        }
    }
    found.sort();
    found
}

/// One log entry as a doc: its frontmatter fields, its body as the text, and its title line.
fn entry_doc(stream: &str, file: &str, path: &Path) -> Result<Doc, Error> {
    let contents = text::read(path).map_err(|failure| io_error(path, &failure))?;
    let document = split(&contents);
    let id = file.strip_suffix(".md").unwrap_or(file).to_string();
    Ok(Doc {
        label: format!("{stream}:{id}"),
        collection: stream.to_string(),
        id,
        path: None,
        title: title_of(&document.body),
        text: document.body,
        fields: document.fields,
    })
}

/// The warning for a path that cannot be read: memex's words, the io kind and the path (design §4.2 and §4.4).
fn warning(path: &Path, failure: &io::Error) -> String {
    format!("memex: warning: {}", io_error(path, failure).message)
}

/// A read failure, naming the path and the io kind and never the OS text (design §4.4).
fn io_error(path: &Path, failure: &io::Error) -> Error {
    Error::io(&format!("cannot read {}", slash(path)), failure)
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn an_unreadable_stream_folder_or_entry_is_a_warning_and_the_rest_loads() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let store = dir.path();
        fs::create_dir_all(store.join("impl")).expect("a stream");
        fs::create_dir_all(store.join("other")).expect("a stream");
        fs::write(store.join("impl/20261014-120000-a.md"), "# A\n").expect("an entry");
        fs::write(store.join("other/20261014-120001-b.md"), "# B\n").expect("an entry");
        fs::write(store.join("other/20261014-120002-c.md"), "# C\n").expect("an entry");
        let open = fs::Permissions::from_mode(0o755);
        let closed = fs::Permissions::from_mode(0o000);
        fs::set_permissions(store.join("impl"), closed.clone()).expect("a mode");
        fs::set_permissions(store.join("other/20261014-120002-c.md"), closed).expect("a mode");
        if fs::read_dir(store.join("impl")).is_ok() {
            // Running as root reads any folder, so there is nothing to check.
            return;
        }
        let mut warnings = Vec::new();
        let docs = load(store, &mut warnings);
        fs::set_permissions(store.join("impl"), open.clone()).expect("restored");
        fs::set_permissions(store.join("other/20261014-120002-c.md"), open).expect("restored");
        let labels: Vec<&str> = docs.iter().map(|doc| doc.label.as_str()).collect();
        assert_eq!(labels, ["other:20261014-120001-b"]);
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(
            warnings[0].contains("impl (PermissionDenied)"),
            "{warnings:?}"
        );
        assert!(
            warnings[1].contains("c.md (PermissionDenied)"),
            "{warnings:?}"
        );
    }
}
