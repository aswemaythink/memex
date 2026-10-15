//! Reads the log store's entries as docs for `show` (design §3.1 and §4.3).
//! Every non-dot `.md` file one folder below the store is one doc, labelled `stream:id`, with its frontmatter as fields and its title from its first line. A stream folder or an entry that cannot be read is a warning, and is skipped (design §4.2); one that is already gone (NotFound) is skipped without a warning (design §4.3 Vanished entries). A name with no label (`is_label_name`) is skipped before any io on it, with no warning (design §4.4).
//! Cases: SHOW-LOG-01, SHOW-LOG-02, SHOW-JSON-03

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::collections::frontmatter::split;
use crate::corpus::Doc;
use crate::error::{self, Error};
use crate::intent::Origin;
use crate::logs::write::title_of;
use crate::root::store_listed;
use crate::text;
use crate::walk::is_label_name;

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
                Ok(Some(doc)) => docs.push(doc),
                Ok(None) => {}
                Err(error) => warnings.push(error::warning(&error.message)),
            }
        }
    }
    docs
}

/// The non-dot entries of `dir` by name, sorted: its folders when `folders` is set, otherwise its `.md` files. A missing
/// `dir` has none. A `dir` that cannot be listed, or an entry whose type cannot be read, adds its warning and is skipped. An
/// entry whose name has no label is skipped before any io on it, with no warning (design §4.4).
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
                note(warnings, dir, &failure);
                continue;
            }
        };
        let file_name = item.file_name();
        if !is_label_name(&file_name) {
            continue;
        }
        // A label name is UTF-8, so its lossy text is exactly the name.
        let name = file_name.to_string_lossy().into_owned();
        let kind = match item.file_type() {
            Ok(kind) => kind,
            Err(failure) => {
                note(warnings, &item.path(), &failure);
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

/// One log entry as a doc: its frontmatter fields, its body as the text, and its title line. An entry that is gone by the time
/// it is read is None, as a concurrent prune leaves it (design §4.3 Vanished entries).
fn entry_doc(stream: &str, file: &str, path: &Path) -> Result<Option<Doc>, Error> {
    let contents = match text::read(path) {
        Ok(contents) => contents,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(failure) => return Err(Error::io_at("cannot read", path, &failure)),
    };
    let document = split(&contents);
    let id = file.strip_suffix(".md").unwrap_or(file).to_string();
    Ok(Some(Doc {
        label: format!("{stream}:{id}"),
        collection: stream.to_string(),
        id,
        path: None,
        title: title_of(&document.body),
        text: document.body,
        fields: document.fields,
        origin: Origin::Other,
    }))
}

/// The warning for a path that cannot be read: memex's words, the io kind and the path (design §4.2 and §4.4).
fn warning(path: &Path, failure: &io::Error) -> String {
    error::warning(&Error::io_at("cannot read", path, failure).message)
}

/// Adds the warning for `path` to `warnings`, unless the failure is NotFound: the path is already gone, as a concurrent prune
/// leaves it (design §4.3 Vanished entries).
fn note(warnings: &mut Vec<String>, path: &Path, failure: &io::Error) {
    if failure.kind() != io::ErrorKind::NotFound {
        warnings.push(warning(path, failure));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn an_entry_or_folder_gone_before_it_is_read_warns_nothing() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let stream = dir.path().join("impl");
        let entry = stream.join("20261014-120000-a.md");
        let mut warnings = Vec::new();
        assert!(entries(&stream, false, &mut warnings).is_empty());
        assert!(matches!(
            entry_doc("impl", "20261014-120000-a.md", &entry),
            Ok(None)
        ));
        assert!(warnings.is_empty(), "{warnings:?}");
        // Any other failure to read an entry is still an error, so NotFound is the only silent one.
        fs::create_dir_all(&entry).expect("a folder where the entry should be");
        assert!(entry_doc("impl", "20261014-120000-a.md", &entry).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_stream_folder_or_entry_is_a_warning_and_the_rest_loads() {
        use std::os::unix::fs::PermissionsExt;

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

    /// A stream folder or an entry whose name has no label is skipped before any io on it: a folder that cannot be listed and
    /// an entry that cannot be read give neither a warning nor a doc (design §4.4). The modes are restored before the asserts.
    #[cfg(unix)]
    #[test]
    fn a_name_with_no_label_is_skipped_before_any_io() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("a temp folder");
        let store = dir.path();
        let stream = store.join("impl");
        fs::create_dir_all(&stream).expect("a stream");
        fs::write(stream.join("20261014-120000-a.md"), "# A\n").expect("an entry");
        let bad_folder = store.join(OsStr::from_bytes(b"bad\x1b"));
        fs::create_dir_all(&bad_folder).expect("a stream");
        fs::write(bad_folder.join("20261014-120001-b.md"), "# B\n").expect("an entry");
        let bad_entry = stream.join("20261014-120002-c\x1b.md");
        fs::write(&bad_entry, "# C\n").expect("an entry");
        let open = fs::Permissions::from_mode(0o755);
        let closed = fs::Permissions::from_mode(0o000);
        fs::set_permissions(&bad_folder, closed.clone()).expect("a mode");
        fs::set_permissions(&bad_entry, closed).expect("a mode");
        let mut warnings = Vec::new();
        let docs = load(store, &mut warnings);
        fs::set_permissions(&bad_folder, open.clone()).expect("restored");
        fs::set_permissions(&bad_entry, open).expect("restored");
        let labels: Vec<&str> = docs.iter().map(|doc| doc.label.as_str()).collect();
        assert_eq!(labels, ["impl:20261014-120000-a"]);
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}
