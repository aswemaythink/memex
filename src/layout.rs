//! Finds a repository's toplevel by listing, and reads its layout as plain files, never by asking git.
//! `scan` finds the toplevel and the root's `.memex/` in one listing of each ancestor. The toplevel bounds the walk for `.memex/` (design §4.1 item 1). The layout (design §4.4 git) is the `.git` entry, its `gitdir:` line, the git dir's `commondir` and its `HEAD`, each read only as a small regular file, so a hostile layout cannot block or flood the read; the listing helpers that match special names against a folder's listing (design §4.4 Name matching) live here too.
//! Cases: DISC-TOP-01, DISC-TOP-02, DISC-TOP-03, DISC-TOP-04, DISC-TOP-05, DISC-TOP-06, DISC-TOP-08, DISC-ROOT-05, DISC-GIT-01, DISC-GIT-02, DISC-GIT-03, DISC-NOGIT-04, DISC-NOGIT-05, DISC-NOGIT-06, DISC-NOGIT-07, INIT-NOGIT-01

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// The toplevel of the repository holding `start` and the nearest folder with a `.memex/` within it (design §4.1 item 1), found by
/// one walk that lists each ancestor once. The toplevel is the nearest folder at or above `start` whose listing has an entry named
/// `.git`, of any kind; with none, the `.memex/` is searched up to the filesystem root. A folder whose listing fails lists nothing.
/// The results are canonical when `start` is.
pub fn scan(start: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    let mut memex = None;
    for dir in start.ancestors() {
        let [git, has_memex] = listed_names(dir, [".git", ".memex"]);
        if memex.is_none() && has_memex && dir.join(".memex").is_dir() {
            memex = Some(dir.to_path_buf());
        }
        if git {
            return (Some(dir.to_path_buf()), memex);
        }
    }
    (None, memex)
}

/// Which of `names` the listing of `dir` holds, each matched exactly, from one read of the folder (design §4.4 Name matching). A
/// folder that does not exist lists nothing, and a failing entry ends the read, so a name that the read did not reach is not held,
/// as `lists` reads it.
fn listed_names<const N: usize>(dir: &Path, names: [&str; N]) -> [bool; N] {
    let mut held = [false; N];
    let Ok(listing) = fs::read_dir(dir) else {
        return held;
    };
    for item in listing {
        let Ok(item) = item else {
            break;
        };
        let file_name = item.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        for (slot, wanted) in held.iter_mut().zip(names) {
            *slot |= name == wanted;
        }
    }
    held
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

/// True when the store's own folder is listed under its parent by its exact name, as a folder, a file or a symlink. The writes
/// and reads that use a listed store follow it (design §4.4 Name matching).
pub fn store_listed(store: &Path) -> io::Result<bool> {
    match (store.parent(), store.file_name().and_then(OsStr::to_str)) {
        (Some(parent), Some(name)) => lists(parent, name),
        _ => Ok(false),
    }
}

/// The largest layout file (`.git`, `commondir`, `HEAD`) memex reads, in bytes. Each holds one short line; git itself refuses a
/// larger `.git` file.
const LAYOUT_FILE_CAP: u64 = 16 * 1024;

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
pub fn git_entry_is_folder(top: &Path) -> bool {
    entry_named(top, ".git")
        .ok()
        .flatten()
        .is_some_and(|entry| entry.path().is_dir())
}

/// True when the `.git` entry of `top` is a file, as a linked worktree's is. Only then does the store map to a main checkout, so a
/// `.git` folder that git skips keeps its root's own store (DISC-TOP-08).
pub fn git_entry_is_file(top: &Path) -> bool {
    entry_named(top, ".git")
        .ok()
        .flatten()
        .is_some_and(|entry| entry.path().is_file())
}

/// The git dir of the repository whose toplevel is `top`: its `.git` folder, or the folder that a `.git` file's `gitdir:` line names,
/// relative to `top`. None when the entry is neither, or the file has no such line.
fn git_dir(top: &Path) -> Option<PathBuf> {
    let entry = entry_named(top, ".git").ok().flatten()?;
    if entry.path().is_dir() {
        return Some(entry.path());
    }
    let line = first_line(&entry.path()).ok()?;
    let target = line.strip_prefix("gitdir:")?.trim();
    (!target.is_empty()).then(|| top.join(target))
}

/// The first line of a layout file, without its line ending. Only a regular file (followed through a symlink) of at most
/// `LAYOUT_FILE_CAP` bytes is read, so a FIFO, a device or a huge file cannot block or exhaust the read.
fn first_line(path: &Path) -> io::Result<String> {
    let meta = fs::metadata(path)?;
    if !meta.is_file() || meta.len() > LAYOUT_FILE_CAP {
        return Err(io::Error::other("not a layout file"));
    }
    let mut text = String::new();
    fs::File::open(path)?
        .take(LAYOUT_FILE_CAP)
        .read_to_string(&mut text)?;
    Ok(text.lines().next().unwrap_or_default().to_string())
}

/// The common dir that a git dir's `commondir` file names, resolved from the git dir (design §4.4 git). `Ok(None)` when the git dir
/// has no `commondir`, as a submodule's or a separate git dir's has not. An error when the git dir cannot be listed, or the file
/// cannot be read or names no folder.
fn common_dir(git_dir: &Path) -> io::Result<Option<PathBuf>> {
    fs::read_dir(git_dir)?;
    let Some(entry) = entry_named(git_dir, "commondir")? else {
        return Ok(None);
    };
    let text = first_line(&entry.path())?;
    let line = text.trim_end();
    if line.is_empty() {
        return Err(io::Error::other("the commondir file is empty"));
    }
    let common = dunce::canonicalize(git_dir.join(line))?;
    if !common.is_dir() {
        return Err(io::Error::other("the commondir file names no folder"));
    }
    Ok(Some(common))
}

/// The branch that the repository's `HEAD` names, read from its git dir (design §4.4 git). None for a detached HEAD, for a HEAD that
/// is not a `ref: refs/heads/<name>` line, and for a name no branch can have.
pub fn head_branch(top: &Path) -> Option<String> {
    let head = first_line(&git_dir(top)?.join("HEAD")).ok()?;
    let name = head
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
        assert_eq!(scan(&sub.join("deeper")).0, Some(sub));
        assert_eq!(scan(&repo).0, Some(repo.clone()));
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
        assert_eq!(scan(&cwd), (Some(repo.clone()), None));
        fs::create_dir_all(repo.join(".memex")).expect("a store in the repo");
        assert_eq!(scan(&cwd), (Some(repo.clone()), Some(repo)));
    }

    #[test]
    fn outside_a_repo_the_walk_reaches_the_filesystem_root() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let cwd = top.join("plain").join("sub");
        fs::create_dir_all(&cwd).expect("a folder with no repo");
        fs::create_dir_all(top.join(".memex")).expect("a store above it");
        assert_eq!(scan(&cwd), (None, Some(top)));
    }

    #[test]
    fn a_store_is_listed_by_its_name_under_its_parent() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let store = dir.path().join(".memex").join("logs");
        assert!(!store_listed(&store).expect("a listing"));
        fs::create_dir_all(&store).expect("a store");
        assert!(store_listed(&store).expect("a listing"));
    }

    #[test]
    fn one_listing_holds_each_name_exactly_and_a_missing_folder_holds_none() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        fs::write(dir.path().join(".git"), b"x").expect("a file");
        fs::create_dir_all(dir.path().join("Config.memex")).expect("a folder");
        assert_eq!(listed_names(dir.path(), [".git", ".memex"]), [true, false]);
        let missing = dir.path().join("missing");
        assert_eq!(listed_names(&missing, [".git", ".memex"]), [false, false]);
    }

    #[test]
    fn a_branch_name_is_not_empty_and_no_part_of_it_starts_with_a_dot() {
        assert!(is_branch_name("main"));
        assert!(is_branch_name("feat/show"));
        assert!(!is_branch_name(""));
        assert!(!is_branch_name(".invalid"));
        assert!(!is_branch_name("feat/.hidden"));
    }

    /// The reads run on another thread, so a read that blocks on the FIFO fails the test after two seconds instead of hanging it.
    #[cfg(unix)]
    #[test]
    fn a_layout_file_that_is_a_fifo_is_not_read() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let made = std::process::Command::new("mkfifo")
            .arg(top.join(".git"))
            .status()
            .expect("mkfifo runs");
        assert!(made.success());
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let unplaced = matches!(layout_of(&top), Layout::Unplaced);
            sender.send((unplaced, head_branch(&top)))
        });
        let outcome = receiver.recv_timeout(std::time::Duration::from_secs(2));
        assert_eq!(outcome.expect("the layout reads return"), (true, None));
    }

    #[cfg(not(unix))]
    #[test]
    fn a_layout_file_that_is_a_fifo_is_not_read() {}

    #[test]
    fn a_layout_file_larger_than_the_cap_is_not_read() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let git = top.join("real.git");
        fs::create_dir_all(&git).expect("a git dir");
        fs::write(git.join("HEAD"), b"ref: refs/heads/main\n").expect("a HEAD");
        let line = format!("gitdir: {}\n", git.display());
        fs::write(top.join(".git"), &line).expect("a git file");
        assert_eq!(head_branch(&top), Some("main".to_string()));
        let padded = format!("{line}{}", " ".repeat(LAYOUT_FILE_CAP as usize));
        fs::write(top.join(".git"), padded).expect("a huge git file");
        assert_eq!(head_branch(&top), None);
        assert!(matches!(layout_of(&top), Layout::Unplaced));
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

    #[test]
    fn a_commondir_naming_a_git_file_is_unplaced_not_the_worktree_itself() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let linked = top.join("main").join(".git").join("worktrees").join("wt");
        fs::create_dir_all(&linked).expect("a linked git dir");
        let wt = top.join("wt");
        fs::create_dir_all(&wt).expect("a worktree");
        fs::write(wt.join(".git"), format!("gitdir: {}\n", linked.display())).expect("a file");
        let own_file = format!("{}\n", wt.join(".git").display());
        fs::write(linked.join("commondir"), own_file).expect("a commondir naming a file");
        assert!(matches!(layout_of(&wt), Layout::Unplaced));
    }
}
