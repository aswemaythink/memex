//! Walks the intent layer: the entries under the root that the ignore rules keep (design §4.1 item 2).
//! Cases: INT-IGNORE-01, INT-IGNORE-02, INT-IGNORE-03, INT-SKIP-01, INT-SKIP-02, SHOW-FILE-01, SHOW-DIR-01, SHOW-ROOT-01

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use ignore::{DirEntry, WalkBuilder};

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
/// not list the root itself.
pub fn walk(root: &Path) -> Vec<Entry> {
    let walker = WalkBuilder::new(root)
        .require_git(false)
        .sort_by_file_name(OsStr::cmp)
        .filter_entry(|entry| !is_skipped(entry))
        .build();
    let mut entries: Vec<Entry> = walker
        .filter_map(Result::ok)
        .filter_map(|entry| to_entry(root, &entry))
        .collect();
    entries.sort_by(|left, right| left.rel.cmp(&right.rel));
    entries
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
