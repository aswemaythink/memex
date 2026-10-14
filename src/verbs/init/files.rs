//! Writes the `.memex/` files (design §3.5b item 1) and holds the file helpers the init modules share.
//! Cases: INIT-FRESH-01, INIT-KEEP-01, INIT-KEEP-02, INIT-RERUN-01, INIT-IO-01, INIT-LINK-01, INIT-LINK-02

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use tempfile::NamedTempFile;

use super::{Action, Report};
use crate::error::Error;
use crate::root::slash;

/// `.memex/.gitignore`: the logs and the cache stay out of git by default (design §4.1 item 4).
const GITIGNORE: &[u8] = b"logs/\ncache/\n";
/// `.memex/config.json`: its one key, at the default retention (design §4.1 item 4).
const CONFIG: &[u8] = b"{\"keep_logs\": \"3d\"}\n";

/// Creates `.memex/` with its two files and `collections/` with its `.gitkeep`, and never overwrites a file that exists
/// (design §3.5b item 1). Each file is recorded as created or kept.
pub fn store(target: &Path, report: &mut Report) -> Result<(), Error> {
    let memex = target.join(".memex");
    let collections = memex.join("collections");
    fs::create_dir_all(&collections)
        .map_err(|failure| io_error("cannot create", &collections, &failure))?;
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

/// True when `dir` lists an entry named exactly `name`. Special names are matched against the listing and never probed with
/// `exists()`, so a case-folding file system behaves like Linux (design §4.4 Name matching). A missing `dir` lists nothing.
pub fn lists(dir: &Path, name: &str) -> Result<bool, Error> {
    let listing = match fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(failure) => return Err(io_error("cannot read", dir, &failure)),
    };
    for item in listing {
        let item = item.map_err(|failure| io_error("cannot read", dir, &failure))?;
        if item.file_name().to_str() == Some(name) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Writes a new file `dir/name` through a temp file in `dir`, so no reader sees part of it. A name that is taken is an error, never
/// an overwrite (design §4.4 Atomic writes).
pub fn create_new(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), Error> {
    let target = dir.join(name);
    let temp = write_temp(dir, bytes, None)?;
    temp.persist_noclobber(&target)
        .map_err(|failed| io_error("cannot create", &target, &failed.error))?;
    Ok(())
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
        .map_err(|failure| io_error("cannot read", path, &failure))?
        .permissions();
    if permissions.readonly() {
        return Err(Error::io_message(format!(
            "cannot update {}: it is read-only",
            slash(path)
        )));
    }
    let temp = write_temp(dir, bytes, Some(permissions))?;
    temp.persist(path)
        .map_err(|failed| io_error("cannot write", path, &failed.error))?;
    Ok(())
}

/// True when the entry at `path` is a symbolic link. The link is not followed, so a link to nothing is a link too. The entry
/// is one that a listing has already named, so this is not an existence probe (design §4.4 Name matching).
pub fn is_link(path: &Path) -> Result<bool, Error> {
    fs::symlink_metadata(path)
        .map(|meta| meta.file_type().is_symlink())
        .map_err(|failure| io_error("cannot read", path, &failure))
}

/// The text of the file at `path`, which must be UTF-8. A file that memex cannot read exactly is never rewritten (design §4.4 Encoding).
pub fn read_text(path: &Path, target: &Path) -> Result<String, Error> {
    let bytes = fs::read(path).map_err(|failure| io_error("cannot read", path, &failure))?;
    String::from_utf8(bytes).map_err(|_| {
        Error::io_message(format!(
            "cannot update {}: it is not UTF-8",
            label(target, path)
        ))
    })
}

/// A temp file in `dir` that holds `bytes`, made with `permissions`, else with those of any new file (on unix 0o666 less the
/// umask, not tempfile's 0o600). It is removed when dropped, unless it is persisted.
pub fn write_temp(
    dir: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<NamedTempFile, Error> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".");
    if let Some(permissions) = permissions.or_else(new_file_permissions) {
        builder.permissions(permissions);
    }
    let mut temp = builder
        .tempfile_in(dir)
        .map_err(|failure| io_error("cannot create a temporary file in", dir, &failure))?;
    temp.write_all(bytes)
        .map_err(|failure| io_error("cannot write", dir, &failure))?;
    Ok(temp)
}

/// The mode a new file gets on unix before the umask, as `File::create` gives it.
#[cfg(unix)]
fn new_file_permissions() -> Option<fs::Permissions> {
    use std::os::unix::fs::PermissionsExt;
    Some(fs::Permissions::from_mode(0o666))
}

/// Windows has no mode to give: a temp file is an ordinary writable file already.
#[cfg(not(unix))]
fn new_file_permissions() -> Option<fs::Permissions> {
    None
}

/// A path as an init line names it: relative to the target when it lies inside it, else as it is, always with `/` separators.
pub fn label(target: &Path, path: &Path) -> String {
    match path.strip_prefix(target) {
        Ok(rel) => slash(rel),
        Err(_) => slash(path),
    }
}

/// An I/O error that names what failed, the path and the io::ErrorKind, and never the OS text (design §4.4 Errors).
pub fn io_error(what: &str, path: &Path, failure: &io::Error) -> Error {
    Error::io(&format!("{what} {}", slash(path)), failure)
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
    fn listings_match_names_exactly_and_a_missing_folder_lists_nothing() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        fs::write(dir.path().join("AGENTS.md"), b"x").expect("a file");
        assert!(lists(dir.path(), "AGENTS.md").expect("a listing"));
        assert!(!lists(dir.path(), "agents.md").expect("a listing"));
        assert!(!lists(&dir.path().join("missing"), "AGENTS.md").expect("a listing"));
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
    fn a_replaced_file_keeps_its_mode_and_a_new_file_gets_the_usual_one() {
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
        fs::write(dir.path().join("plain"), b"x").expect("a plain file");
        assert_eq!(
            mode(&dir.path().join("CLAUDE.md")),
            mode(&dir.path().join("plain"))
        );
    }
}
