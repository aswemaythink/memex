//! Walks the intent layer: the entries under the root that the ignore rules keep (design §4.1 item 2).
//! A name that is not UTF-8, or that holds a control character, has no label (`is_label_name`), so the walk never enters it and gives it no warning (design §4.4).
//! Cases: INT-IGNORE-01, INT-IGNORE-02, INT-IGNORE-03, INT-SKIP-01, INT-SKIP-02, INT-NAME-03, SHOW-FILE-01, SHOW-DIR-01, SHOW-ROOT-01, SHOW-IO-02, DISC-IGNORE-01

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;

use ignore::{DirEntry, WalkBuilder, WalkState};

use crate::error::{self, Error};
use crate::par;

/// One file or folder of the intent layer.
pub struct Entry {
    /// The root-relative path, with `/` separators.
    pub rel: String,
    /// The path to read the entry from.
    pub path: PathBuf,
    /// True for a folder, false for a file.
    pub is_dir: bool,
}

/// A walk failure that names a path: the path, the kind of its io error, and the warning it gives (design §4.2).
struct Failure {
    /// The path the failure names, as the walk joined it from the root.
    path: PathBuf,
    /// The kind of the io error under it.
    kind: io::ErrorKind,
    /// The `memex: warning:` line for it.
    warning: String,
}

/// Every file and folder under `root` that the ignore rules keep, sorted by label. The walk honours `.gitignore` and
/// `.ignore`, skips hidden entries, `node_modules` and `target` folders, symlinks, and names that are not UTF-8 or hold
/// a control character. It does not list the root itself. A folder that cannot be listed gives no entries here;
/// `walk_warned` says so.
pub fn walk(root: &Path) -> Vec<Entry> {
    walk_warned(root, &mut Vec::new(), &mut BTreeMap::new())
}

/// Like `walk`, and each folder under `root` that the walk cannot read adds its warning to `warnings`, `memex: warning: cannot
/// read <path> (<ErrorKind>)` (design §4.2), in the order of a depth-first walk. The rest of the tree is still walked. A folder
/// that the walk could not list is also recorded in `unlistable`, by its root-relative path with the kind of its failure, so
/// that `show` can fail a request for it (design §3.3). The walk runs on several threads, so its items are collected, then
/// ordered by path and by label.
pub fn walk_warned(
    root: &Path,
    warnings: &mut Vec<String>,
    unlistable: &mut BTreeMap<String, io::ErrorKind>,
) -> Vec<Entry> {
    let (sender, items) = mpsc::channel();
    let mut builder = WalkBuilder::new(root);
    builder
        .require_git(false)
        .threads(par::threads())
        .filter_entry(|entry| !is_skipped(entry));
    builder.build_parallel().run(|| {
        let sender = sender.clone();
        Box::new(move |item: Result<DirEntry, ignore::Error>| {
            if let Some(found) = found_of(root, item) {
                // The receiver is read after every walker thread has ended, so a send cannot fail.
                let _ = sender.send(found);
            }
            WalkState::Continue
        })
    });
    drop(sender);
    let (mut entries, mut failures) = (Vec::new(), Vec::new());
    for found in items {
        match found {
            Ok(entry) => entries.push(entry),
            Err(failure) => failures.push(failure),
        }
    }
    // A path orders its components, so this puts each warning where the depth-first walk gave it: a folder's before its children's.
    failures.sort_by(|left, right| left.path.cmp(&right.path));
    // Each path is walked once, so its label is unique and an unstable sort gives the one order.
    entries.sort_unstable_by(|left, right| left.rel.cmp(&right.rel));
    for failure in &failures {
        if let Some(rel) = unlistable_rel(root, &entries, failure) {
            unlistable.insert(rel, failure.kind);
        }
    }
    warnings.extend(failures.into_iter().map(|failure| failure.warning));
    entries
}

/// The root-relative path of a failure that names a folder the walk could not list: the root itself, or a folder that the walk
/// kept. None for any other failure, such as one that names an ignore file.
fn unlistable_rel(root: &Path, entries: &[Entry], failure: &Failure) -> Option<String> {
    let rel = rel_string(failure.path.strip_prefix(root).ok()?)?;
    let is_folder = rel.is_empty()
        || entries
            .binary_search_by(|entry| entry.rel.cmp(&rel))
            .is_ok_and(|at| entries[at].is_dir);
    is_folder.then_some(rel)
}

/// What one walk item gives: its entry, or its failure. None for an item that is neither.
fn found_of(root: &Path, item: Result<DirEntry, ignore::Error>) -> Option<Result<Entry, Failure>> {
    match item {
        Ok(entry) => to_entry(root, &entry).map(Ok),
        Err(error) => walk_warning(&error).map(Err),
    }
}

/// The failure that a walk error names, with its warning (design §4.2). None for any other error, such as an ignore file
/// that does not parse.
fn walk_warning(error: &ignore::Error) -> Option<Failure> {
    let (path, failure) = named_failure(error)?;
    let message = Error::io_at("cannot read", path, failure).message;
    Some(Failure {
        path: path.to_path_buf(),
        kind: failure.kind(),
        warning: error::warning(&message),
    })
}

/// The path that a walk error names, and the io failure under it. The walk wraps a failure in its path and its depth, in
/// either order, so each wrapper is looked through. None when there is no io failure, or no path for it.
fn named_failure(error: &ignore::Error) -> Option<(&Path, &io::Error)> {
    let mut named = None;
    let mut inner = error;
    loop {
        match inner {
            ignore::Error::WithPath { path, err } => {
                named = named.or(Some(path.as_path()));
                inner = err.as_ref();
            }
            ignore::Error::WithDepth { err, .. } => inner = err.as_ref(),
            ignore::Error::Io(failure) => return named.map(|path| (path, failure)),
            _ => return None,
        }
    }
}

/// The `/`-separated text of a root-relative path: `""` for the root, None when a part has no label (`is_label_name`), or
/// when the path climbs out.
pub fn rel_string(rel: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) if is_label_name(part) => parts.push(part.to_str()?),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(parts.join("/"))
}

/// True for a name that has a label: UTF-8, with no control character (U+0000–U+001F or U+007F, design §4.4). The walk never
/// enters a name that fails this, and the log reader, prune and the manifest listing test it before any io on the entry.
/// `is_ascii_control` is exactly that set, and none of its bytes occurs inside a longer UTF-8 character, so a byte scan decides it.
pub fn is_label_name(name: &OsStr) -> bool {
    name.to_str()
        .is_some_and(|text| !text.bytes().any(|byte| byte.is_ascii_control()))
}

/// True for an entry below the root that the walk must not enter: a name with no label, or a `node_modules` or `target`
/// folder (design §4.1 item 2). A skipped folder is never listed, so nothing under it gives an entry or a warning.
fn is_skipped(entry: &DirEntry) -> bool {
    entry.depth() > 0 && (!is_label_name(entry.file_name()) || is_build_folder(entry))
}

/// True for a `node_modules` or `target` folder.
fn is_build_folder(entry: &DirEntry) -> bool {
    entry.file_type().is_some_and(|kind| kind.is_dir())
        && matches!(entry.file_name().to_str(), Some("node_modules" | "target"))
}

/// The walk entry for a folder or a regular file below the root. Symlinks, the root and unlabelable paths give None.
fn to_entry(root: &Path, entry: &DirEntry) -> Option<Entry> {
    if entry.depth() == 0 {
        return None;
    }
    let kind = entry.file_type()?;
    if !(kind.is_dir() || kind.is_file()) {
        return None;
    }
    let rel = rel_string(entry.path().strip_prefix(root).ok()?)?;
    Some(Entry {
        rel,
        path: entry.path().to_path_buf(),
        is_dir: kind.is_dir(),
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};

    use super::{is_label_name, rel_string};

    #[test]
    fn a_path_with_a_part_that_is_not_utf8_has_no_label() {
        assert_eq!(rel_string(Path::new("a/b.md")).as_deref(), Some("a/b.md"));
        assert_eq!(rel_string(&not_utf8()), None);
    }

    /// A part with a control character has no label: U+0000–U+001F and U+007F (design §4.4). Space and `~` are not control
    /// characters, and U+0080 is outside that set, so those parts keep their labels.
    #[test]
    fn a_path_with_a_control_character_has_no_label() {
        assert_eq!(
            rel_string(Path::new("a/b c~.md")).as_deref(),
            Some("a/b c~.md")
        );
        assert_eq!(
            rel_string(Path::new("a/\u{80}.md")).as_deref(),
            Some("a/\u{80}.md")
        );
        assert_eq!(rel_string(Path::new("a/nul\u{0}.md")), None);
        assert_eq!(rel_string(Path::new("a/unit\u{1f}.md")), None);
        assert_eq!(rel_string(Path::new("a/del\u{7f}.md")), None);
        assert_eq!(rel_string(Path::new("a\nb.rs")), None);
        assert_eq!(rel_string(Path::new("tab\there/README.md")), None);
    }

    /// The predicate that the walk, the log reader, prune and the manifest listing share: UTF-8 with no control character.
    /// Space, `~` and U+0080 pass, and so does a non-ASCII letter; a non-UTF-8 name fails.
    #[test]
    fn a_name_has_a_label_only_when_it_is_utf8_with_no_control_character() {
        for name in ["b c~.md", "\u{80}.md", "é.md"] {
            assert!(is_label_name(OsStr::new(name)), "{name:?}");
        }
        for name in [
            "nul\u{0}.md",
            "unit\u{1f}.md",
            "del\u{7f}.md",
            "a\nb.rs",
            "tab\there",
        ] {
            assert!(!is_label_name(OsStr::new(name)), "{name:?}");
        }
        let invalid = not_utf8();
        assert!(!is_label_name(invalid.file_name().expect("a name")));
    }

    /// A folder whose name has no label is never entered, so one that cannot be listed gives no warning and no entry, whether
    /// its name holds a control character or is not UTF-8 (design §4.4). The folders are restored before the asserts.
    #[cfg(unix)]
    #[test]
    fn a_folder_with_no_label_is_never_entered_so_it_gives_no_warning() {
        use std::collections::BTreeMap;
        use std::fs;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::PermissionsExt;

        use super::walk_warned;
        let dir = tempfile::tempdir().expect("a temp folder");
        let escaped = dir.path().join("esc\u{1b}");
        fs::create_dir(&escaped).expect("a folder");
        let mut folders = vec![escaped];
        // APFS refuses a name that is not UTF-8, so that folder is made only where the file system allows one.
        let invalid = dir.path().join(OsStr::from_bytes(b"bad\xff"));
        if fs::create_dir(&invalid).is_ok() {
            folders.push(invalid);
        }
        fs::write(dir.path().join("ok.rs"), "//! Ok.\n").expect("a file");
        for folder in &folders {
            fs::write(folder.join("README.md"), "# Locked\n").expect("a file");
            fs::set_permissions(folder, fs::Permissions::from_mode(0o000)).expect("a mode");
        }
        let open = fs::Permissions::from_mode(0o755);
        if fs::read_dir(&folders[0]).is_ok() {
            // Running as root lists a folder whatever its mode, so there is nothing to check.
            for folder in &folders {
                fs::set_permissions(folder, open.clone()).expect("restored");
            }
            return;
        }
        let (mut warnings, mut unlistable) = (Vec::new(), BTreeMap::new());
        let entries = walk_warned(dir.path(), &mut warnings, &mut unlistable);
        for folder in &folders {
            fs::set_permissions(folder, open.clone()).expect("restored");
        }
        let labels: Vec<&str> = entries.iter().map(|entry| entry.rel.as_str()).collect();
        assert_eq!(labels, ["ok.rs"]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(unlistable.is_empty(), "{unlistable:?}");
    }

    /// A folder that cannot be listed is one warning, and the files beside it are still walked (design §4.2). The folder is
    /// also recorded as unlistable, so that `show` can fail a request for it (design §3.3, SHOW-IO-02).
    #[cfg(unix)]
    #[test]
    fn a_folder_that_cannot_be_listed_is_a_warning_and_the_rest_is_walked() {
        use std::collections::BTreeMap;
        use std::fs;
        use std::io;
        use std::os::unix::fs::PermissionsExt;

        use super::walk_warned;
        let dir = tempfile::tempdir().expect("a temp folder");
        let locked = dir.path().join("locked");
        fs::create_dir(&locked).expect("a folder");
        fs::write(dir.path().join("ok.rs"), "//! Ok.\n").expect("a file");
        let open = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("a mode");
        if fs::read_dir(&locked).is_ok() {
            // Running as root lists a folder whatever its mode, so there is nothing to check.
            fs::set_permissions(&locked, open).expect("restored");
            return;
        }
        let (mut warnings, mut unlistable) = (Vec::new(), BTreeMap::new());
        let entries = walk_warned(dir.path(), &mut warnings, &mut unlistable);
        fs::set_permissions(&locked, open).expect("restored");
        assert!(entries.iter().any(|entry| entry.rel == "ok.rs"));
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].starts_with("memex: warning: cannot read ")
                && warnings[0].ends_with("locked (PermissionDenied)"),
            "{warnings:?}"
        );
        assert_eq!(unlistable.len(), 1, "{unlistable:?}");
        assert_eq!(
            unlistable.get("locked"),
            Some(&io::ErrorKind::PermissionDenied)
        );
    }

    /// A path whose last part is not UTF-8: raw bytes on unix, an unpaired surrogate on Windows.
    #[cfg(unix)]
    fn not_utf8() -> PathBuf {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(OsStr::from_bytes(b"a/\xff.md"))
    }

    /// A path whose last part is not UTF-8: raw bytes on unix, an unpaired surrogate on Windows.
    #[cfg(windows)]
    fn not_utf8() -> PathBuf {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let mut wide: Vec<u16> = "a/".encode_utf16().collect();
        wide.push(0xD800);
        wide.extend(".md".encode_utf16());
        PathBuf::from(OsString::from_wide(&wide))
    }
}
