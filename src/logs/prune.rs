//! Prunes the log store after every write (design §4.3).
//! It deletes each entry older than keep_logs by its filename stamp, and removes the stream folders that its deletions empty. The store itself may be a symlink, which is followed; no symlink below it is, and hidden names and anything that is not a log entry are never touched. A path that is already gone (NotFound) is skipped without a warning (design §4.3 Vanished entries).
//! Cases: LOG-PRUNE-01, LOG-PRUNE-02, LOG-PRUNE-03, LOG-PRUNE-04, LOG-PRUNE-05, LOG-PRUNE-06, LOG-PRUNE-07, LOG-PRUNE-08, LOG-PRUNE-09, LOG-PRUNE-10, LOG-PRUNE-11, LOG-PRUNE-12, LOG-PRUNE-13, LOG-PRUNE-14, LOG-PRUNE-15, LOG-PRUNE-17

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::clock::{self, Timestamp};
use crate::error;
use crate::root::slash;

/// Deletes the entries of `store` whose stamps are more than `keep` seconds before `now`. A failure is a warning on stderr, never an error, so the entry that was just written stays and `log` still succeeds.
pub fn run(store: &Path, keep: i64, now: Timestamp) {
    // The store is followed, as a user may point `.memex/logs` elsewhere; its streams are described, never followed.
    if !is_folder(store, fs::metadata(store)) {
        return;
    }
    for stream in names(store) {
        if is_folder(&stream, fs::symlink_metadata(&stream)) {
            prune_stream(&stream, keep, now);
        }
    }
}

/// True when `meta`, read from `path`, is a folder. Read with `symlink_metadata`, a symlink is not a folder, so nothing behind it is deleted.
/// A path that cannot be read is a warning, and counts as not a folder.
fn is_folder(path: &Path, meta: io::Result<fs::Metadata>) -> bool {
    match meta {
        Ok(meta) => meta.is_dir(),
        Err(failure) => {
            warn("cannot read the folder", path, &failure);
            false
        }
    }
}

/// Deletes the expired entries of one stream folder, then removes the folder if those deletions emptied it.
fn prune_stream(dir: &Path, keep: i64, now: Timestamp) {
    let mut deleted = 0;
    for path in names(dir) {
        if expired(&path, keep, now) && delete(&path) {
            deleted += 1;
        }
    }
    if deleted > 0 {
        remove_if_empty(dir);
    }
}

/// The paths of the entries of `dir` that are not hidden and have UTF-8 names, sorted. Only names are read here.
/// A folder that cannot be listed is a warning, and gives no names.
fn names(dir: &Path) -> Vec<PathBuf> {
    let listing = match fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(failure) => {
            warn("cannot read the folder", dir, &failure);
            return Vec::new();
        }
    };
    let mut found = Vec::new();
    for item in listing {
        match item {
            Ok(item) if is_visible(&item.file_name()) => found.push(item.path()),
            Ok(_) => {}
            Err(failure) => warn("cannot read the folder", dir, &failure),
        }
    }
    found.sort();
    found
}

/// True for a name that is UTF-8 and does not start with a dot. Hidden and non-UTF-8 names are never log entries or stream folders.
fn is_visible(name: &OsStr) -> bool {
    name.to_str().is_some_and(|text| !text.starts_with('.'))
}

/// True when the name of `path` is a log entry name whose stamp is more than `keep` seconds before `now`. Only the name is read.
fn expired(path: &Path, keep: i64, now: Timestamp) -> bool {
    let stamp = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(log_stamp);
    stamp.is_some_and(|stamp| now.seconds_since(stamp) > keep)
}

/// The stamp of a log entry name, `YYYYMMDD-HHMMSS-<slug>.md` (design §4.3). The name needs a slug of at least one character, and the stamp must be a real UTC date-time. Otherwise None.
fn log_stamp(name: &str) -> Option<Timestamp> {
    let stamp = name.get(..clock::STAMP_LEN)?;
    let slug = name
        .get(clock::STAMP_LEN..)?
        .strip_prefix('-')?
        .strip_suffix(".md")?;
    if slug.is_empty() || slug.contains('\n') {
        return None;
    }
    clock::parse_stamp(stamp)
}

/// Deletes one expired entry when it is a regular file, and reports whether it did. The entry is described, never followed, so a symlink or a folder is left alone. A failed read or delete is a warning.
fn delete(path: &Path) -> bool {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(failure) => {
            warn("cannot read the entry", path, &failure);
            return false;
        }
    };
    if !meta.is_file() {
        return false;
    }
    match fs::remove_file(path) {
        Ok(()) => true,
        Err(failure) => {
            warn("cannot delete the entry", path, &failure);
            false
        }
    }
}

/// Removes a stream folder that holds nothing now. A folder with anything left in it is kept as it is.
fn remove_if_empty(dir: &Path) {
    let mut rest = match fs::read_dir(dir) {
        Ok(rest) => rest,
        Err(failure) => {
            warn("cannot read the folder", dir, &failure);
            return;
        }
    };
    if rest.next().is_some() {
        return;
    }
    settle_removal(dir, fs::remove_dir(dir));
}

/// Reports a failed removal of a stream folder that was empty when it was checked. DirectoryNotEmpty is silent: a writer has
/// landed a file in the folder since, so the folder is in use and stays. This is a ruling, not yet in design §4.3.
fn settle_removal(dir: &Path, outcome: io::Result<()>) {
    match outcome {
        Ok(()) => {}
        Err(failure) if failure.kind() == io::ErrorKind::DirectoryNotEmpty => {}
        Err(failure) => warn("cannot remove the empty folder", dir, &failure),
    }
}

/// Prints one warning line on stderr: `memex: warning: prune: <message> (<ErrorKind>) <path>`. A NotFound is never printed: the
/// path is already gone, as a concurrent prune leaves it (design §4.3 Vanished entries). A failed write is ignored, as there is
/// nowhere left to report it.
fn warn(message: &str, path: &Path, failure: &io::Error) {
    if failure.kind() == io::ErrorKind::NotFound {
        return;
    }
    emit(error::warning(&format!(
        "prune: {message} ({:?}) {}",
        failure.kind(),
        slash(path)
    )));
}

/// Puts one warning line on stderr. The tests keep the lines in `PRINTED` instead.
#[cfg(not(test))]
fn emit(line: String) {
    error::warn(&[line]);
}

#[cfg(test)]
fn emit(line: String) {
    PRINTED.with(|printed| printed.borrow_mut().push(line));
}

#[cfg(test)]
thread_local! {
    /// The warning lines that `emit` kept on this thread, so a test can check which paths warn.
    static PRINTED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_names_need_a_stamp_a_slug_and_md() {
        let stamp = clock::parse_stamp("20261013-120000");
        assert_eq!(log_stamp("20261013-120000-x.md"), stamp);
        assert_eq!(log_stamp("20261013-120000-a-b.c.md"), stamp);
        let refused = [
            "20261013-120000-.md",
            "20261013-120000.md",
            "20261013_120000-x.md",
            "20261013-120000-x.txt",
            ".20261013-120000-x.md",
            "2026101-120000-x.md",
            "20261013-120000-x\nd.md",
            "",
        ];
        for name in refused {
            assert_eq!(log_stamp(name), None, "{name:?}");
        }
    }

    #[test]
    fn an_entry_exactly_keep_seconds_old_is_kept() {
        let now = clock::parse_stamp("20261013-120000").expect("a stamp");
        let keep = 3 * 86_400;
        assert!(!expired(Path::new("impl/20261010-120000-x.md"), keep, now));
        assert!(expired(Path::new("impl/20261010-115959-x.md"), keep, now));
        assert!(!expired(Path::new("impl/20261020-120000-x.md"), keep, now));
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_delete_is_a_warning_and_the_entry_stays() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().expect("a temporary folder");
        let stream = root.path().join("impl");
        fs::create_dir(&stream).expect("a stream folder");
        let entry = stream.join("20261009-120000-old.md");
        fs::write(&entry, b"old\n").expect("an old entry");
        let now = clock::parse_stamp("20261013-120000").expect("a stamp");
        let restore = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&stream, fs::Permissions::from_mode(0o555)).expect("read-only");
        // A user who may still write to the read-only folder, such as root, deletes the entry; there is nothing to test then.
        if fs::write(stream.join("probe"), b"").is_ok() {
            fs::set_permissions(&stream, restore).expect("restored");
            return;
        }
        prune_stream(&stream, 3 * 86_400, now);
        fs::set_permissions(&stream, restore).expect("restored");
        assert!(
            entry.is_file(),
            "a failed delete must leave the entry in place"
        );
    }

    #[test]
    fn a_vanished_stream_folder_or_entry_warns_nothing() {
        PRINTED.with(|printed| printed.borrow_mut().clear());
        let root = tempfile::tempdir().expect("a temporary folder");
        let stream = root.path().join("impl");
        let entry = stream.join("20261009-120000-old.md");
        let now = clock::parse_stamp("20261013-120000").expect("a stamp");
        let keep = 3 * 86_400;
        assert!(names(&stream).is_empty());
        assert!(!delete(&entry));
        remove_if_empty(&stream);
        prune_stream(&stream, keep, now);
        run(&root.path().join("no-store"), keep, now);
        // The NotFound that a remove of a path another prune took would give.
        let gone = io::Error::from(io::ErrorKind::NotFound);
        warn("cannot delete the entry", &entry, &gone);
        warn("cannot remove the empty folder", &stream, &gone);
        let printed = PRINTED.with(|printed| printed.borrow().clone());
        assert!(printed.is_empty(), "{printed:?}");
    }

    #[test]
    fn any_other_failure_still_warns() {
        PRINTED.with(|printed| printed.borrow_mut().clear());
        let entry = Path::new("impl/20261009-120000-old.md");
        let refused = io::Error::from(io::ErrorKind::PermissionDenied);
        warn("cannot delete the entry", entry, &refused);
        let printed = PRINTED.with(|printed| printed.borrow().clone());
        assert_eq!(printed.len(), 1, "{printed:?}");
        assert!(
            printed[0].contains("prune: cannot delete the entry (PermissionDenied)"),
            "{printed:?}"
        );
    }

    #[test]
    fn a_folder_a_writer_filled_after_the_check_is_kept_silently() {
        PRINTED.with(|printed| printed.borrow_mut().clear());
        let root = tempfile::tempdir().expect("a temporary folder");
        let stream = root.path().join("impl");
        fs::create_dir(&stream).expect("a stream folder");
        // What a writer leaves in the folder between the check and the removal: its dot-prefixed temp file.
        fs::write(stream.join(".writer-temp"), b"").expect("a writer's temp file");
        let refused = fs::remove_dir(&stream).expect_err("a folder with a file in it");
        assert_eq!(refused.kind(), io::ErrorKind::DirectoryNotEmpty);
        settle_removal(&stream, Err(refused));
        assert!(stream.is_dir(), "the folder stays");
        let printed = PRINTED.with(|printed| printed.borrow().clone());
        assert!(printed.is_empty(), "{printed:?}");
        // Any other failure to remove the folder still warns.
        settle_removal(
            &stream,
            Err(io::Error::from(io::ErrorKind::PermissionDenied)),
        );
        assert_eq!(PRINTED.with(|printed| printed.borrow().len()), 1);
    }
}
