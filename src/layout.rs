//! Finds the toplevel of a repository by listing folders for a `.git` entry, never by asking git.
//! The toplevel bounds the walk for `.memex/` (design §4.1 item 1), and the listing helpers that match special names against a folder's listing (design §4.4 Name matching) live here too.
//! Cases: DISC-TOP-01, DISC-TOP-02, DISC-TOP-03, DISC-TOP-04, DISC-TOP-05, DISC-TOP-06, DISC-ROOT-05, INIT-NOGIT-01

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The toplevel of the repository holding `start`: the nearest folder at or above it whose listing has an entry named `.git`, of
/// any kind (design §4.1 item 1). None when no `.git` lies above `start`. A folder whose listing fails lists nothing, as
/// `holds_memex` reads it. The result is canonical when `start` is.
pub fn toplevel(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| lists(dir, ".git").unwrap_or(false))
        .map(Path::to_path_buf)
}

/// The nearest folder from `cwd` up to `bound`, inclusive, that holds a `.memex/`. With no `bound` the walk runs to the
/// filesystem root. For the bound to be reached, `cwd` and `bound` must both be canonical.
pub fn nearest_within(cwd: &Path, bound: Option<&Path>) -> Option<PathBuf> {
    for dir in cwd.ancestors() {
        if holds_memex(dir) {
            return Some(dir.to_path_buf());
        }
        if Some(dir) == bound {
            break;
        }
    }
    None
}

/// True when `dir` lists a folder named exactly `.memex`. The name is matched against the listing, never probed on a guessed
/// path, so a case-insensitive file system does not match another spelling (design §4.4 Name matching).
pub fn holds_memex(dir: &Path) -> bool {
    lists(dir, ".memex").unwrap_or(false) && dir.join(".memex").is_dir()
}

/// The entry named exactly `name` in `dir`'s listing, if there is one. A folder that does not exist lists nothing. The name is
/// matched against the listing and never probed (design §4.4 Name matching).
pub fn entry_named(dir: &Path, name: &str) -> io::Result<Option<fs::DirEntry>> {
    let listing = match fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(failure) => return Err(failure),
    };
    for item in listing {
        let item = item?;
        if item.file_name().to_str() == Some(name) {
            return Ok(Some(item));
        }
    }
    Ok(None)
}

/// True when `dir` lists an entry named exactly `name`, of any kind. A folder that does not exist lists nothing. Names inside
/// `.memex/` are matched this way and never probed with `exists()` (design §4.4 Name matching).
pub fn lists(dir: &Path, name: &str) -> io::Result<bool> {
    entry_named(dir, name).map(|entry| entry.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listings_match_names_exactly_and_a_missing_folder_lists_nothing() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        fs::write(dir.path().join("Config.json"), b"{}").expect("a file");
        assert!(lists(dir.path(), "Config.json").expect("a listing"));
        assert!(!lists(dir.path(), "config.json").expect("a listing"));
        assert!(!lists(&dir.path().join("missing"), "config.json").expect("a listing"));
    }

    #[test]
    fn the_toplevel_is_the_nearest_folder_that_lists_a_git_entry_of_any_kind() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let repo = top.join("repo");
        let sub = repo.join("sub");
        fs::create_dir_all(sub.join("deeper")).expect("folders below the repo");
        fs::write(repo.join(".git"), b"gitdir: missing\n").expect("a git file");
        fs::create_dir_all(sub.join(".git")).expect("a git folder");
        assert_eq!(toplevel(&sub.join("deeper")), Some(sub));
        assert_eq!(toplevel(&repo), Some(repo.clone()));
    }

    #[test]
    fn the_walk_stops_at_the_toplevel() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let repo = top.join("repo");
        let cwd = repo.join("sub");
        fs::create_dir_all(top.join(".memex")).expect("a folder above the repo");
        fs::create_dir_all(&cwd).expect("a folder inside the repo");
        fs::create_dir_all(repo.join(".git")).expect("a git folder");
        let bound = toplevel(&cwd);
        assert_eq!(nearest_within(&cwd, bound.as_deref()), None);
        assert_eq!(nearest_within(&cwd, None), Some(top.clone()));
        fs::create_dir_all(repo.join(".memex")).expect("a store in the repo");
        assert_eq!(nearest_within(&cwd, bound.as_deref()), Some(repo));
    }
}
