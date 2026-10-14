//! Reads one walked file into its docs: a README, a CASES.md's cases, or a header (design §2).
//! Cases: HDR-CAP-01, HDR-CAP-04, HDR-BIN-01, HDR-EXT-01, INT-BOM-01, INT-LINE-01

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use super::{cases, header, lead, readme};
use crate::corpus::{Doc, HEADERS};
use crate::error::Error;
use crate::root::slash;
use crate::walk::Entry;

/// The most bytes of a file that header detection reads (design §2.1).
const HEAD_BYTES: u64 = 8 * 1024;

/// The docs one walked file gives: its README, the cases of a CASES.md, or its header. A file that cannot be read is an error.
pub fn file_docs(entry: &Entry) -> Result<Vec<Doc>, Error> {
    match entry.path.file_name().and_then(|name| name.to_str()) {
        Some("README.md") => readme::doc(&entry.rel, &entry.path).map(|doc| vec![doc]),
        Some("CASES.md") => cases::read(&entry.rel, &entry.path),
        _ => Ok(header_doc(entry)?.into_iter().collect()),
    }
}

/// The doc of a file's header, when the file has one. A file with no comment family has no header, so it is only opened, which
/// still reports one that cannot be read (design §4.2).
fn header_doc(entry: &Entry) -> Result<Option<Doc>, Error> {
    if !header::has_family(&entry.path) {
        File::open(&entry.path).map_err(|failure| io_error(&entry.path, &failure))?;
        return Ok(None);
    }
    let head = read_head(&entry.path)?;
    let Some(text) = header::header(&entry.path, &head) else {
        return Ok(None);
    };
    Ok(Some(Doc {
        label: entry.rel.clone(),
        collection: HEADERS.to_string(),
        id: entry.rel.clone(),
        path: Some(entry.rel.clone()),
        title: lead(&text),
        text,
        fields: Vec::new(),
    }))
}

/// The first bytes of a file, at most `HEAD_BYTES`.
fn read_head(path: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(path).map_err(|failure| io_error(path, &failure))?;
    let mut head = Vec::new();
    file.take(HEAD_BYTES)
        .read_to_end(&mut head)
        .map_err(|failure| io_error(path, &failure))?;
    Ok(head)
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

    /// An entry at `path`, with the label a walk gives it.
    fn entry_at(path: std::path::PathBuf, rel: &str) -> Entry {
        Entry {
            rel: rel.to_string(),
            path,
            is_dir: false,
        }
    }

    #[test]
    fn an_unreadable_header_file_is_an_error_naming_its_path() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("locked.rs");
        fs::write(&path, "//! Locked.\n").expect("a file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read(&path).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let error = file_docs(&entry_at(path, "locked.rs")).expect_err("an unreadable header file");
        assert!(
            error.message.ends_with("locked.rs (PermissionDenied)"),
            "{}",
            error.message
        );
    }

    #[test]
    fn an_unreadable_file_with_no_header_family_is_still_an_error() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("locked.zzz");
        fs::write(&path, "data\n").expect("a file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read(&path).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let error = file_docs(&entry_at(path, "locked.zzz")).expect_err("an unreadable file");
        assert!(
            error.message.ends_with("locked.zzz (PermissionDenied)"),
            "{}",
            error.message
        );
    }
}
