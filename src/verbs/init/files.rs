//! Writes the `.memex/` files (design §3.5b item 1) and holds the file helpers the init modules share.
//! Its config.json holds `max_logs_mb` at its default of 10 MiB (design §4.1 item 4).
//! Cases: INIT-FRESH-01, INIT-KEEP-01, INIT-KEEP-02, INIT-RERUN-01, INIT-IO-01, INIT-LINK-01, INIT-LINK-02, PORT-MODE-01

use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use super::{Action, Report};
use crate::atomic;
use crate::error::Error;
use crate::logs::write::GITIGNORE;
use crate::root::{self, slash};

/// `.memex/config.json`: its one key, `max_logs_mb` at its default of 10 MiB (design §4.1 item 4). Age pruning stays off.
const CONFIG: &[u8] = b"{\"max_logs_mb\": 10}\n";

/// Creates `.memex/` with its two files and `collections/` with its `.gitkeep`, and never overwrites a file that exists
/// (design §3.5b item 1). Each file is recorded as created or kept.
pub fn store(target: &Path, report: &mut Report) -> Result<(), Error> {
    let memex = target.join(".memex");
    let collections = memex.join("collections");
    fs::create_dir_all(&collections)
        .map_err(|failure| Error::io_at("cannot create", &collections, &failure))?;
    // The logs and the cache stay out of git by default, with the bytes `log` writes beside a new store (design §4.1 item 4).
    place(target, &memex, ".gitignore", GITIGNORE, report)?;
    place(target, &memex, "config.json", CONFIG, report)?;
    place(target, &collections, ".gitkeep", b"", report)
}

/// Creates `dir/name` with `bytes` when no entry of that name is listed, and leaves it when one is.
fn place(
    target: &Path,
    dir: &Path,
    name: &str,
    bytes: &[u8],
    report: &mut Report,
) -> Result<(), Error> {
    let action = if lists(dir, name)? {
        Action::Kept
    } else {
        create_new(dir, name, bytes)?;
        Action::Created
    };
    report.action(action, &label(target, &dir.join(name)));
    Ok(())
}

/// True when `dir` lists an entry named exactly `name`, as `root::lists` reads it, with a failed listing as the I/O error that
/// names `dir`. Special names are never probed with `exists()` (design §4.4 Name matching).
pub fn lists(dir: &Path, name: &str) -> Result<bool, Error> {
    root::lists(dir, name).map_err(|failure| Error::io_at("cannot read", dir, &failure))
}

/// Writes a new file `dir/name` through a temp file in `dir`, so no reader sees part of it. A name that is taken is an error, never
/// an overwrite (design §4.4 Atomic writes).
pub fn create_new(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), Error> {
    let target = dir.join(name);
    match atomic::persist_new(dir, name, bytes) {
        Ok(Some(_)) => Ok(()),
        // A name taken since init's check is an error, never an overwrite: the file that holds it stays as it is (design §3.5b).
        Ok(None) => Err(Error::io_message(format!(
            "cannot create {}: another process created it while init ran, or a name that differs only in case holds it",
            slash(&target)
        ))),
        Err(failed) => Err(failed.into_error(dir, name)),
    }
}

/// Replaces the file at `path` with `bytes`, through a temp file beside it that takes the file's permissions, so the user's mode
/// survives. A read-only file is refused, never replaced. A symlink is refused and never written through, so the link and the file
/// it names stay as they are (design §3.5b item 2). Callers check for links first and warn (design §4.4 Atomic writes).
pub fn replace(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if is_link(path)? {
        return Err(Error::io_message(format!(
            "cannot update {}: it is a symlink",
            slash(path)
        )));
    }
    let Some(dir) = path.parent() else {
        return Err(Error::io_message(format!(
            "cannot write {}: it has no folder",
            slash(path)
        )));
    };
    let permissions = fs::metadata(path)
        .map_err(|failure| Error::io_at("cannot read", path, &failure))?
        .permissions();
    if permissions.readonly() {
        return Err(Error::io_message(format!(
            "cannot update {}: it is read-only",
            slash(path)
        )));
    }
    let temp = write_temp(dir, bytes, Some(permissions))?;
    temp.persist(path)
        .map_err(|failed| Error::io_at("cannot write", path, &failed.error))?;
    Ok(())
}

/// True when the entry at `path` is a symbolic link. The link is not followed, so a link to nothing is a link too. The entry
/// is one that a listing has already named, so this is not an existence probe (design §4.4 Name matching).
pub fn is_link(path: &Path) -> Result<bool, Error> {
    fs::symlink_metadata(path)
        .map(|meta| meta.file_type().is_symlink())
        .map_err(|failure| Error::io_at("cannot read", path, &failure))
}

/// The text of the file at `path`, which must be UTF-8. A file that memex cannot read exactly is never rewritten (design §4.4 Encoding).
pub fn read_text(path: &Path, target: &Path) -> Result<String, Error> {
    let bytes = fs::read(path).map_err(|failure| Error::io_at("cannot read", path, &failure))?;
    String::from_utf8(bytes).map_err(|_| {
        Error::io_message(format!(
            "cannot update {}: it is not UTF-8",
            label(target, path)
        ))
    })
}

/// A temp file in `dir` that holds `bytes`, made with `permissions`, else with those of any new file (`atomic::temp_in`). It is
/// removed when dropped, unless it is persisted.
pub fn write_temp(
    dir: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<NamedTempFile, Error> {
    let mut temp = atomic::temp_in(dir, permissions)
        .map_err(|failure| Error::io_at("cannot create a temporary file in", dir, &failure))?;
    temp.write_all(bytes)
        .map_err(|failure| Error::io_at("cannot write", dir, &failure))?;
    Ok(temp)
}

/// A path as an init line names it: relative to the target when it lies inside it, else as it is, always with `/` separators.
pub fn label(target: &Path, path: &Path) -> String {
    match path.strip_prefix(target) {
        Ok(rel) => slash(rel),
        Err(_) => slash(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_relative_inside_the_target_and_plain_outside_it() {
        let target = Path::new("work/repo");
        assert_eq!(
            label(target, Path::new("work/repo/.memex/config.json")),
            ".memex/config.json"
        );
        assert_eq!(
            label(target, Path::new("elsewhere/memex")),
            "elsewhere/memex"
        );
    }

    #[test]
    fn a_read_only_file_is_refused_and_left_as_it_is() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, b"old").expect("a file");
        let mut permissions = fs::metadata(&path).expect("metadata").permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).expect("read-only");
        assert!(replace(&path, b"new").is_err());
        assert_eq!(fs::read(&path).expect("the file"), b"old");
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_is_refused_and_neither_it_nor_its_target_changes() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let real = dir.path().join("shared.md");
        fs::write(&real, b"old").expect("a file");
        let link = dir.path().join("AGENTS.md");
        std::os::unix::fs::symlink(&real, &link).expect("a symlink");
        assert!(replace(&link, b"new").is_err());
        assert!(is_link(&link).expect("a link"));
        assert_eq!(fs::read(&real).expect("the target"), b"old");
    }

    #[cfg(unix)]
    #[test]
    fn a_replaced_file_keeps_its_mode_and_a_new_file_gets_0644_before_the_umask() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("a temporary folder");
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, b"old").expect("a file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).expect("a mode");
        replace(&path, b"new").expect("a replace");
        let mode = |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode(&path), 0o640);
        assert_eq!(fs::read(&path).expect("the file"), b"new");
        create_new(dir.path(), "CLAUDE.md", b"x").expect("a new file");
        // A file that File::create makes gets 0o666 less the umask, which recovers the umask of this run.
        fs::write(dir.path().join("plain"), b"x").expect("a plain file");
        let umask = 0o666 & !mode(&dir.path().join("plain"));
        assert_eq!(mode(&dir.path().join("CLAUDE.md")), 0o644 & !umask);
    }

    #[test]
    fn a_name_taken_after_the_check_gets_the_race_message_and_keeps_its_file() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, b"theirs").expect("a file");
        let error = create_new(dir.path(), "CLAUDE.md", b"ours").expect_err("a taken name");
        assert_eq!(error.code.as_str(), "io");
        assert_eq!(
            error.message,
            format!(
                "cannot create {}: another process created it while init ran, or a name that differs only in case holds it",
                slash(&path)
            )
        );
        assert_eq!(fs::read(&path).expect("the file"), b"theirs");
    }
}
