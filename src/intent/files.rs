//! Reads one walked file into its docs: a README, a Markdown file's docs, or a header (design §2).
//! Cases: HDR-CAP-01, HDR-CAP-04, HDR-BIN-01, HDR-EXT-01, INT-BOM-01, INT-LINE-01, HDR-READ-01, HDR-READ-02, COLL-README-01, COLL-UNREAD-01

use super::{front, header, lead, readme, Origin};
use crate::corpus::{Doc, HEADERS};
use crate::error::Error;
use crate::root::slash;
use crate::walk::Entry;

/// What one walked file gives: its docs, and the warnings that it gives without losing them (design §4.2).
#[derive(Debug, Default)]
pub struct Found {
    /// The docs of the file, in the order they are read.
    pub docs: Vec<Doc>,
    /// The warnings of the file, each without its `memex: warning: ` prefix, which the corpus adds.
    pub warnings: Vec<String>,
}

/// The docs of one walked entry: none for a folder, and those of its file otherwise (design §2 and §4.2).
pub fn read_entry(entry: &Entry) -> Result<Found, Error> {
    if entry.is_dir {
        return Ok(Found::default());
    }
    file_docs(entry)
}

/// The docs one walked file gives (design §2.2): a README's doc, the docs of a Markdown file whose frontmatter names a collection,
/// or a file's header. A file that cannot be read is an error, except a file with no comment family, which is never opened
/// (design §4.2).
pub fn file_docs(entry: &Entry) -> Result<Found, Error> {
    match entry.path.file_name().and_then(|name| name.to_str()) {
        Some("README.md") => readme_found(entry),
        Some(name) if name.ends_with(".md") => front::markdown(&entry.rel, &entry.path),
        _ => Ok(Found {
            docs: header_doc(entry)?.into_iter().collect(),
            warnings: Vec::new(),
        }),
    }
}

/// The doc of a README, with the warning that its `memex:` key is ignored when it has one (design §2.2). The README is read once
/// when its head holds all of it.
fn readme_found(entry: &Entry) -> Result<Found, Error> {
    let head = front::Head::read(&entry.path)?;
    let text = head.text(&entry.path)?;
    let mut warnings = Vec::new();
    if front::readme_has_key(&head, &text) {
        warnings.push(format!(
            "{} is a README, which belongs to the graph; its memex: key is ignored — move that content into its own file",
            slash(&entry.path)
        ));
    }
    Ok(Found {
        docs: vec![readme::doc(&entry.rel, text)],
        warnings,
    })
}

/// The doc of a file's header, when the file has one. A file with no comment family has no header and is never opened, so it
/// gives no warning even when it cannot be read (design §4.2). A file named by its family, such as `Makefile`, is opened.
fn header_doc(entry: &Entry) -> Result<Option<Doc>, Error> {
    if !header::has_family(&entry.path) {
        return Ok(None);
    }
    let head = header::read_head(&entry.path)
        .map_err(|failure| Error::io_at("cannot read", &entry.path, &failure))?;
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
        origin: Origin::Graph,
    }))
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
    fn an_unreadable_readme_is_an_error_naming_its_path() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("README.md");
        fs::write(&path, "# Locked\n\nLocked.\n").expect("a file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read(&path).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let error = file_docs(&entry_at(path, "README.md")).expect_err("an unreadable README");
        assert!(
            error.message.ends_with("README.md (PermissionDenied)"),
            "{}",
            error.message
        );
    }

    #[test]
    fn an_unreadable_markdown_file_is_an_error_naming_its_path() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("notes.md");
        fs::write(&path, "# Notes\n").expect("a file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read(&path).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let error = file_docs(&entry_at(path, "notes.md")).expect_err("an unreadable .md");
        assert!(
            error.message.ends_with("notes.md (PermissionDenied)"),
            "{}",
            error.message
        );
    }

    #[test]
    fn an_unreadable_file_with_no_header_family_is_never_opened() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("locked.zzz");
        fs::write(&path, "data\n").expect("a file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read(&path).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let found = file_docs(&entry_at(path, "locked.zzz"))
            .expect("a file with no comment family is never opened, so it is no error");
        assert!(found.docs.is_empty());
    }

    #[test]
    fn an_unreadable_file_named_by_its_family_is_still_an_error() {
        for name in ["Makefile", "Dockerfile"] {
            let dir = tempfile::tempdir().expect("a temp folder");
            let path = dir.path().join(name);
            fs::write(&path, "# Build.\n").expect("a file");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("a mode");
            if fs::read(&path).is_ok() {
                // Running as root reads any file, so there is nothing to check.
                return;
            }
            let error =
                file_docs(&entry_at(path, name)).expect_err("an unreadable file with a family");
            assert!(
                error
                    .message
                    .ends_with(&format!("{name} (PermissionDenied)")),
                "{}",
                error.message
            );
        }
    }
}
