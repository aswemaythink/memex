//! Reads one walked file into its docs: a README, a Markdown file's docs, or a header (design §2).
//! Cases: HDR-CAP-01, HDR-CAP-04, HDR-BIN-01, HDR-EXT-01, INT-BOM-01, INT-LINE-01, HDR-READ-01, HDR-READ-02, COLL-README-01, COLL-UNREAD-01, SHOW-MDCHILD-01, SHOW-MDCHILD-02

use super::header::{self, lines};
use super::{front, lead, readme, Origin};
use crate::corpus::{Doc, HEADERS};
use crate::error::Error;
use crate::root::slash;
use crate::walk::Entry;

/// What one walked file gives: its docs, the warnings that it gives without losing them (design §4.2), and the title of a Markdown
/// file's text (design §3.3).
#[derive(Debug, Default)]
pub struct Found {
    /// The docs of the file, in the order they are read.
    pub docs: Vec<Doc>,
    /// The warnings of the file, each without its `memex: warning: ` prefix, which the corpus adds.
    pub warnings: Vec<String>,
    /// The title of a Markdown file's text, which its folder's child line shows; None for any other file.
    pub title: Option<String>,
    /// The line facts of a file with a comment family, when the read asked for them (`audit`, design §3.5d Lines); else None.
    pub head: Option<lines::Head>,
}

/// The docs of one walked entry: none for a folder, and those of its file otherwise (design §2 and §4.2). With `lines`, a file with
/// a comment family also gives the line facts of its head, for `audit`.
pub fn read_entry(entry: &Entry, lines: bool) -> Result<Found, Error> {
    if entry.is_dir {
        return Ok(Found::default());
    }
    file_docs(entry, lines)
}

/// The docs one walked file gives (design §2.2): a README's doc, the docs of a Markdown file whose frontmatter names a collection,
/// or a file's header. A file that cannot be read is an error, except a file with no comment family, which is never opened
/// (design §4.2). With `lines`, a file with a comment family also gives the line facts of its head.
pub fn file_docs(entry: &Entry, lines: bool) -> Result<Found, Error> {
    match entry.path.file_name().and_then(|name| name.to_str()) {
        Some("README.md") => readme_found(entry),
        Some(name) if name.ends_with(".md") => front::markdown(&entry.rel, &entry.path),
        _ => header_found(entry, lines),
    }
}

/// The doc of a README, with the warning that its `memex:` key is ignored when it has one (design §2.2). The README is read whole,
/// in one open, and its head is taken from that read.
fn readme_found(entry: &Entry) -> Result<Found, Error> {
    let (head, text) = front::Head::read_all(&entry.path)?;
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
        ..Found::default()
    })
}

/// The doc of a file's header, when the file has one, and with `lines` the line facts of its head. A file with no comment family
/// has no header and is never opened, so it gives no warning even when it cannot be read (design §4.2). A file named by its
/// family, such as `Makefile`, is opened.
fn header_found(entry: &Entry, lines: bool) -> Result<Found, Error> {
    if !header::has_family(&entry.path) {
        return Ok(Found::default());
    }
    let bytes = header::read_head(&entry.path)
        .map_err(|failure| Error::io_at("cannot read", &entry.path, &failure))?;
    let (text, head) = if lines {
        header::with_lines(&entry.path, &bytes)
    } else {
        (header::header(&entry.path, &bytes), None)
    };
    Ok(Found {
        docs: text
            .map(|text| header_doc(entry, text))
            .into_iter()
            .collect(),
        head,
        ..Found::default()
    })
}

/// The doc of a file's header, labelled by the file's path (design §3.1).
fn header_doc(entry: &Entry, text: String) -> Doc {
    Doc {
        label: entry.rel.clone(),
        collection: HEADERS.to_string(),
        id: entry.rel.clone(),
        path: Some(entry.rel.clone()),
        title: lead(&text),
        text,
        fields: Vec::new(),
        origin: Origin::Graph,
    }
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
        let error =
            file_docs(&entry_at(path, "locked.rs"), false).expect_err("an unreadable header file");
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
        let error =
            file_docs(&entry_at(path, "README.md"), false).expect_err("an unreadable README");
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
        let error = file_docs(&entry_at(path, "notes.md"), false).expect_err("an unreadable .md");
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
        let found = file_docs(&entry_at(path, "locked.zzz"), false)
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
            let error = file_docs(&entry_at(path, name), false)
                .expect_err("an unreadable file with a family");
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
