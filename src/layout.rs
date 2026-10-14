//! Finds a repository's toplevel by listing, and reads its layout as plain files, never by asking git.
//! The toplevel bounds the walk for `.memex/` (design §4.1 item 1). The layout (design §4.4 git) is the `.git` entry, its `gitdir:` line, the git dir's `commondir` and its `HEAD`; the listing helpers that match special names against a folder's listing (design §4.4 Name matching) live here too.
//! Cases: DISC-TOP-01, DISC-TOP-02, DISC-TOP-03, DISC-TOP-04, DISC-TOP-05, DISC-TOP-06, DISC-ROOT-05, DISC-GIT-01, DISC-GIT-02, DISC-GIT-03, DISC-NOGIT-04, DISC-NOGIT-05, DISC-NOGIT-06, DISC-NOGIT-07, INIT-NOGIT-01

use std::ffi::OsStr;
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

/// What the layout of a repository says about its log store, when git gives no answer (design §4.4 git).
pub enum Layout {
    /// A main checkout (a `.git` folder), a submodule or a separate git dir (a `.git` file whose git dir has no `commondir`): the
    /// store is the root's own.
    Own,
    /// A linked worktree whose common dir is a `.git` folder, so the main checkout holds the store. The value is that folder.
    Linked(PathBuf),
    /// A linked worktree the layout cannot place: its common dir has another name (a bare repo's), or an entry it names cannot be
    /// read.
    Unplaced,
}

/// Reads the layout of the repository whose toplevel is `top`, from plain files only: the `.git` entry, the `gitdir:` line of a
/// `.git` file, and the git dir's `commondir` (design §4.4 git). Refs, `packed-refs`, objects, the index and config are never read.
pub fn layout_of(top: &Path) -> Layout {
    if git_entry_is_folder(top) {
        return Layout::Own;
    }
    let Some(dir) = git_dir(top) else {
        return Layout::Unplaced;
    };
    match common_dir(&dir) {
        Ok(None) => Layout::Own,
        Ok(Some(common)) if common.file_name() == Some(OsStr::new(".git")) => {
            Layout::Linked(common)
        }
        Ok(Some(_)) | Err(_) => Layout::Unplaced,
    }
}

/// True when the `.git` entry of `top` is a folder, which is a main checkout (design §4.4 git). The name was matched by the listing
/// that found `top`.
fn git_entry_is_folder(top: &Path) -> bool {
    entry_named(top, ".git")
        .ok()
        .flatten()
        .is_some_and(|entry| entry.path().is_dir())
}

/// The git dir of the repository whose toplevel is `top`: its `.git` folder, or the folder that a `.git` file's `gitdir:` line names,
/// relative to `top`. None when the entry is neither, or the file has no such line.
pub fn git_dir(top: &Path) -> Option<PathBuf> {
    let entry = entry_named(top, ".git").ok().flatten()?;
    if entry.path().is_dir() {
        return Some(entry.path());
    }
    let text = fs::read_to_string(entry.path()).ok()?;
    let target = text.lines().next()?.strip_prefix("gitdir:")?.trim();
    (!target.is_empty()).then(|| top.join(target))
}

/// The common dir that a git dir's `commondir` file names, resolved from the git dir (design §4.4 git). `Ok(None)` when the git dir
/// has no `commondir`, as a submodule's or a separate git dir's has not. An error when the git dir cannot be listed, or the file
/// cannot be read or names no folder.
fn common_dir(git_dir: &Path) -> io::Result<Option<PathBuf>> {
    fs::read_dir(git_dir)?;
    let Some(entry) = entry_named(git_dir, "commondir")? else {
        return Ok(None);
    };
    let text = fs::read_to_string(entry.path())?;
    let line = text.lines().next().map(str::trim_end).unwrap_or_default();
    if line.is_empty() {
        return Err(io::Error::other("the commondir file is empty"));
    }
    dunce::canonicalize(git_dir.join(line)).map(Some)
}

/// The branch that the repository's `HEAD` names, read from its git dir (design §4.4 git). None for a detached HEAD, for a HEAD that
/// is not a `ref: refs/heads/<name>` line, and for a name no branch can have.
pub fn head_branch(top: &Path) -> Option<String> {
    let head = fs::read_to_string(git_dir(top)?.join("HEAD")).ok()?;
    let name = head
        .lines()
        .next()?
        .strip_prefix("ref:")?
        .trim()
        .strip_prefix("refs/heads/")?;
    is_branch_name(name).then(|| name.to_string())
}

/// True when `name` could name a branch: it is not empty, and no `/`-separated part starts with a dot, which is how reftable's
/// placeholder `refs/heads/.invalid` fails.
fn is_branch_name(name: &str) -> bool {
    !name.is_empty() && !name.split('/').any(|part| part.starts_with('.'))
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

    #[test]
    fn a_branch_name_is_not_empty_and_no_part_of_it_starts_with_a_dot() {
        assert!(is_branch_name("main"));
        assert!(is_branch_name("feat/show"));
        assert!(!is_branch_name(""));
        assert!(!is_branch_name(".invalid"));
        assert!(!is_branch_name("feat/.hidden"));
    }

    #[test]
    fn a_linked_worktree_maps_to_its_main_checkout_and_a_dangling_git_file_is_unplaced() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let common = top.join("main").join(".git");
        let linked = common.join("worktrees").join("wt");
        fs::create_dir_all(&linked).expect("a linked git dir");
        fs::write(linked.join("commondir"), b"../..\n").expect("a commondir");
        fs::write(linked.join("HEAD"), b"ref: refs/heads/feat\r\n").expect("a HEAD");
        let wt = top.join("wt");
        fs::create_dir_all(&wt).expect("a worktree");
        fs::write(wt.join(".git"), format!("gitdir: {}\n", linked.display())).expect("a file");
        assert!(matches!(layout_of(&wt), Layout::Linked(ref found) if *found == common));
        assert_eq!(head_branch(&wt), Some("feat".to_string()));
        fs::write(wt.join(".git"), b"gitdir: missing\n").expect("a dangling file");
        assert!(matches!(layout_of(&wt), Layout::Unplaced));
        assert_eq!(head_branch(&wt), None);
    }
}
