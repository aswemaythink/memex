//! Writes the `.memex/` files (design §3.5b item 1) and holds the file helpers the init modules share.
//! Cases: INIT-FRESH-01, INIT-KEEP-01, INIT-KEEP-02, INIT-RERUN-01

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
    let temp = write_temp(dir, bytes)?;
    temp.persist_noclobber(&target)
        .map_err(|failed| io_error("cannot create", &target, &failed.error))?;
    Ok(())
}

/// Replaces the file at `path` with `bytes`, through a temp file beside its target. A symlink is followed, so the file it names is
/// the one written, and the link stays as it is (design §4.4 Atomic writes).
pub fn replace(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let real =
        dunce::canonicalize(path).map_err(|failure| io_error("cannot resolve", path, &failure))?;
    let Some(dir) = real.parent() else {
        return Err(Error::io_message(format!(
            "cannot write {}: it has no folder",
            slash(&real)
        )));
    };
    let temp = write_temp(dir, bytes)?;
    temp.persist(&real)
        .map_err(|failed| io_error("cannot write", &real, &failed.error))?;
    Ok(())
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

/// A temp file in `dir` that holds `bytes`. It is removed when dropped, unless it is persisted.
pub fn write_temp(dir: &Path, bytes: &[u8]) -> Result<NamedTempFile, Error> {
    let mut temp = tempfile::Builder::new()
        .prefix(".")
        .tempfile_in(dir)
        .map_err(|failure| io_error("cannot create a temporary file in", dir, &failure))?;
    temp.write_all(bytes)
        .map_err(|failure| io_error("cannot write", dir, &failure))?;
    Ok(temp)
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
}
