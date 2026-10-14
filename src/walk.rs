//! Walks the intent layer: the entries under the root that the ignore rules keep (design §4.1 item 2).
//! Cases: INT-IGNORE-01, INT-IGNORE-02, INT-IGNORE-03, INT-SKIP-01, INT-SKIP-02, INT-NAME-03, SHOW-FILE-01, SHOW-DIR-01, SHOW-ROOT-01

use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;

use ignore::{DirEntry, WalkBuilder, WalkState};

use crate::error::Error;
use crate::par;
use crate::root::slash;

/// One file or folder of the intent layer.
pub struct Entry {
    /// The root-relative path, with `/` separators.
    pub rel: String,
    /// The path to read the entry from.
    pub path: PathBuf,
    /// True for a folder, false for a file.
    pub is_dir: bool,
}

/// Every file and folder under `root` that the ignore rules keep, sorted by label. The walk honours `.gitignore` and
/// `.ignore`, skips hidden entries, `node_modules` and `target` folders, symlinks and names that are not UTF-8, and does
/// not list the root itself. A folder that cannot be listed gives no entries here; `walk_warned` says so.
pub fn walk(root: &Path) -> Vec<Entry> {
    walk_warned(root, &mut Vec::new())
}

/// Like `walk`, and each folder under `root` that the walk cannot read adds its warning to `warnings`, `memex: warning: cannot
/// read <path> (<ErrorKind>)` (design §4.2), in the order of a depth-first walk. The rest of the tree is still walked. The walk
/// runs on several threads, so its items are collected, then ordered by path and by label.
pub fn walk_warned(root: &Path, warnings: &mut Vec<String>) -> Vec<Entry> {
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
    failures.sort_by(|left, right| left.0.cmp(&right.0));
    warnings.extend(failures.into_iter().map(|(_, text)| text));
    entries.sort_by(|left, right| left.rel.cmp(&right.rel));
    entries
}

/// What one walk item gives: its entry, or its warning with the path that it names. None for an item that is neither.
fn found_of(
    root: &Path,
    item: Result<DirEntry, ignore::Error>,
) -> Option<Result<Entry, (PathBuf, String)>> {
    match item {
        Ok(entry) => to_entry(root, &entry).map(Ok),
        Err(error) => walk_warning(&error).map(Err),
    }
}

/// The path that a walk error names, with the warning for it (design §4.2). None for any other error, such as an ignore file
/// that does not parse.
fn walk_warning(error: &ignore::Error) -> Option<(PathBuf, String)> {
    let (path, failure) = named_failure(error)?;
    let message = Error::io(&format!("cannot read {}", slash(path)), failure).message;
    Some((path.to_path_buf(), format!("memex: warning: {message}")))
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

/// The `/`-separated text of a root-relative path: `""` for the root, None when a part is not UTF-8 or the path climbs out.
pub fn rel_string(rel: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_str()?),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(parts.join("/"))
}

/// True for a folder below the root that the walk must not enter: `node_modules` or `target`.
fn is_skipped(entry: &DirEntry) -> bool {
    entry.depth() > 0
        && entry.file_type().is_some_and(|kind| kind.is_dir())
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
    use std::path::{Path, PathBuf};

    use super::rel_string;

    #[test]
    fn a_path_with_a_part_that_is_not_utf8_has_no_label() {
        assert_eq!(rel_string(Path::new("a/b.md")).as_deref(), Some("a/b.md"));
        assert_eq!(rel_string(&not_utf8()), None);
    }

    /// A folder that cannot be listed is one warning, and the files beside it are still walked (design §4.2).
    #[cfg(unix)]
    #[test]
    fn a_folder_that_cannot_be_listed_is_a_warning_and_the_rest_is_walked() {
        use std::fs;
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
        let mut warnings = Vec::new();
        let entries = walk_warned(dir.path(), &mut warnings);
        fs::set_permissions(&locked, open).expect("restored");
        assert!(entries.iter().any(|entry| entry.rel == "ok.rs"));
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].starts_with("memex: warning: cannot read ")
                && warnings[0].ends_with("locked (PermissionDenied)"),
            "{warnings:?}"
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
