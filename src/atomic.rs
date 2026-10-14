//! Writes a new file through a dot-prefixed temp file, never over an old one (design §4.4).
//! It is the one writer that log and init share. The temp file sits in the folder of the file it becomes and is made with an
//! ordinary file's mode, so a persisted file is one.
//! Cases: PORT-MODE-01, INIT-FRESH-01, LOG-WRITE-08, LOG-WRITE-15

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;

/// The step of writing a new file that failed, with the I/O error of that step. Each caller names the step in its own words.
#[derive(Debug)]
pub enum Failure {
    /// The temp file could not be made in the folder.
    Create(io::Error),
    /// The bytes could not be written into the temp file.
    Write(io::Error),
    /// The temp file could not be persisted under the name, and the name is not taken.
    Persist(io::Error),
}

/// Writes `bytes` to `dir/name` as a new file and returns its path. A name that is taken gives None and writes nothing, and the
/// entry that holds the name is left as it is.
pub fn persist_new(dir: &Path, name: &str, bytes: &[u8]) -> Result<Option<PathBuf>, Failure> {
    let target = dir.join(name);
    let mut temp = temp_in(dir, None).map_err(Failure::Create)?;
    temp.write_all(bytes).map_err(Failure::Write)?;
    match temp.persist_noclobber(&target) {
        Ok(_) => Ok(Some(target)),
        Err(failed) if taken(&target, &failed.error) => Ok(None),
        Err(failed) => Err(Failure::Persist(failed.error)),
    }
}

/// True when a failed persist means the name is taken: the error says so, or something exists at the name in any form,
/// which `symlink_metadata` sees without following it. Windows reports a folder there as a different error (design §4.4).
fn taken(target: &Path, failure: &io::Error) -> bool {
    failure.kind() == io::ErrorKind::AlreadyExists || fs::symlink_metadata(target).is_ok()
}

/// A dot-prefixed temp file in `dir`, made with `permissions`, else with those of any new file: 0o644 before the umask on unix,
/// not tempfile's private 0o600. It is removed when dropped, unless it is persisted.
pub fn temp_in(dir: &Path, permissions: Option<fs::Permissions>) -> io::Result<NamedTempFile> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".");
    if let Some(permissions) = permissions.or_else(new_file_mode) {
        builder.permissions(permissions);
    }
    builder.tempfile_in(dir)
}

/// The mode a new file gets before the umask: 0o644 on unix (design §4.4 File modes).
#[cfg(unix)]
fn new_file_mode() -> Option<fs::Permissions> {
    use std::os::unix::fs::PermissionsExt;
    Some(fs::Permissions::from_mode(0o644))
}

/// Windows has no mode to set: a temp file is an ordinary writable file already.
#[cfg(not(unix))]
fn new_file_mode() -> Option<fs::Permissions> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names in `dir`, sorted.
    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .expect("the folder")
            .map(|item| {
                item.expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn a_taken_name_gives_none_and_leaves_the_file_as_it_was() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        fs::write(dir.path().join("entry.md"), b"old\n").expect("a file");
        let outcome = persist_new(dir.path(), "entry.md", b"new\n").expect("no failure");
        assert_eq!(outcome, None);
        assert_eq!(
            fs::read(dir.path().join("entry.md")).expect("the file"),
            b"old\n"
        );
        assert_eq!(
            names(dir.path()),
            vec!["entry.md"],
            "the temp file is gone too"
        );
    }

    #[test]
    fn a_folder_at_the_name_is_taken_and_left_as_it_was() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        fs::create_dir(dir.path().join("entry.md")).expect("a folder");
        let outcome = persist_new(dir.path(), "entry.md", b"new\n").expect("no failure");
        assert_eq!(outcome, None);
        assert!(dir.path().join("entry.md").is_dir());
        assert_eq!(names(dir.path()), vec!["entry.md"]);
    }

    #[test]
    fn a_new_name_is_written_and_no_temp_file_is_left() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let written = persist_new(dir.path(), "entry.md", b"new\n").expect("no failure");
        assert_eq!(written, Some(dir.path().join("entry.md")));
        assert_eq!(
            fs::read(dir.path().join("entry.md")).expect("the file"),
            b"new\n"
        );
        assert_eq!(names(dir.path()), vec!["entry.md"]);
    }

    #[test]
    fn a_folder_that_is_not_there_fails_at_the_create_step() {
        let root = tempfile::tempdir().expect("a temporary folder");
        let missing = root.path().join("gone");
        assert!(matches!(
            persist_new(&missing, "entry.md", b"x\n"),
            Err(Failure::Create(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_new_file_gets_0644_before_the_umask_and_not_the_private_one() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("a temporary folder");
        persist_new(dir.path(), "entry.md", b"x\n")
            .expect("no failure")
            .expect("the entry");
        fs::write(dir.path().join("plain"), b"x\n").expect("a plain file");
        let mode = |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
        // A file that File::create makes gets 0o666 less the umask, which recovers the umask of this run.
        let umask = 0o666 & !mode(&dir.path().join("plain"));
        assert_eq!(mode(&dir.path().join("entry.md")), 0o644 & !umask);
    }
}
