//! Finds where `log` writes, per design §4.1: the root, the folder that owns the store, and the store.
//! The root is the nearest `.memex/` above the cwd, bounded by its toplevel, which the listing finds (layout.rs); else the toplevel, else the cwd. The read verbs use the same root. git answers only for the store's place and HEAD.
//! Cases: DISC-ROOT-01, DISC-ROOT-02, DISC-ROOT-03, DISC-ROOT-04, DISC-ROOT-05, DISC-WT-01, DISC-WT-02, DISC-BARE-01, DISC-GIT-01, DISC-NOGIT-01, DISC-NOGIT-02, DISC-NOGIT-03, DISC-TOP-01, DISC-TOP-02, DISC-TOP-03, DISC-TOP-04, DISC-TOP-05, DISC-TOP-06, LOG-WRITE-06, LOG-WRITE-10

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::Error;
use crate::layout::{entry_named, holds_memex, nearest_within, toplevel};

pub use crate::layout::lists;

/// Where `log` writes: the root, the folder whose `.memex/` owns the store, and the store itself.
pub struct Place {
    /// The folder whose `.memex/` holds config.json and the store: the root, or its counterpart in the main checkout.
    pub owner: PathBuf,
    /// The log store: `<owner>/.memex/logs`, or `<git-common-dir>/memex/logs` for a bare repo.
    pub store: PathBuf,
    /// True when the store is inside a bare repo's git dir, so no `.gitignore` is written beside it.
    pub in_git_dir: bool,
    /// The toplevel of a linked worktree, which the entry's `worktree` field names. None in the main checkout.
    pub worktree: Option<PathBuf>,
}

/// The branch and short commit of HEAD. Each is None when it cannot be known: no git, no commits,
/// or, for the branch, a detached HEAD.
pub struct Head {
    pub branch: Option<String>,
    pub commit: Option<String>,
}

/// What git said about the cwd. Only present inside a repository.
struct Git {
    /// The absolute git common dir: the main checkout's `.git`, or a bare repo's own folder.
    common_dir: PathBuf,
    /// True when the common dir is a bare repository.
    bare: bool,
    /// True when the cwd is in a linked worktree, whose git dir differs from the common dir.
    linked: bool,
}

/// Finds the place `log` writes from `cwd`. A missing `.memex/` is the `no_root` error (exit 3).
pub fn locate(cwd: &Path) -> Result<Place, Error> {
    let canonical = dunce::canonicalize(cwd)
        .map_err(|failure| Error::io_at("cannot resolve", cwd, &failure))?;
    // The toplevel comes from the listing, so it is the same with git missing, failing or present (design §4.1 item 1).
    let top = toplevel(&canonical);
    let found = nearest_within(&canonical, top.as_deref());
    let has_memex = found.is_some();
    let git = Git::probe(&canonical);
    let worktree = git.as_ref().filter(|git| git.linked).and(top.clone());
    let root = found.or_else(|| top.clone()).unwrap_or(canonical);
    match git {
        Some(git) if git.bare => bare_place(root, &git, has_memex, worktree),
        git => checkout_place(root, top.as_deref(), git.as_ref(), has_memex, worktree),
    }
}

/// `locate` for `log`. Without git, a linked worktree is refused, since its entry would be lost with the worktree (design §4.4
/// git). The nearest `.git` entry at or above `cwd` decides it, and git is asked only when that entry is a file.
pub fn locate_for_log(cwd: &Path) -> Result<Place, Error> {
    let here = resolved(cwd);
    if nearest_git_is_file(&here) && git_missing() {
        return Err(Error::io_message(NO_GIT_IN_WORKTREE.to_string()));
    }
    locate(cwd)
}

/// The refusal of `log` in a linked worktree that git cannot be asked about (design §4.4 git).
const NO_GIT_IN_WORKTREE: &str = "cannot find the main checkout's log store: git is not on PATH (this folder is a linked worktree); put git on PATH";

/// The branch and short commit of HEAD, as git sees them from `cwd`.
pub fn head(cwd: &Path) -> Head {
    let branch =
        git_line(cwd, &["rev-parse", "--abbrev-ref", "HEAD"]).filter(|name| name != "HEAD");
    let commit = git_line(cwd, &["rev-parse", "--short=7", "HEAD"]);
    Head { branch, commit }
}

/// A path as labels and JSON write it: `/` separators on every OS, and no change on Unix.
pub fn slash(path: &Path) -> String {
    let text = path.to_string_lossy();
    if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.into_owned()
    }
}

/// The store of a bare repo: inside its git dir, owned by the root, which must hold a `.memex/`.
fn bare_place(
    root: PathBuf,
    git: &Git,
    has_memex: bool,
    worktree: Option<PathBuf>,
) -> Result<Place, Error> {
    if !has_memex {
        return Err(Error::no_root());
    }
    Ok(Place {
        store: git.common_dir.join("memex").join("logs"),
        owner: root,
        in_git_dir: true,
        worktree,
    })
}

/// The store of an ordinary folder or checkout. `log` needs a `.memex/` in the root or the store owner.
fn checkout_place(
    root: PathBuf,
    top: Option<&Path>,
    git: Option<&Git>,
    has_memex: bool,
    worktree: Option<PathBuf>,
) -> Result<Place, Error> {
    let owner = store_owner(&root, top, git);
    if !has_memex && !holds_memex(&owner) {
        return Err(Error::no_root());
    }
    let store = owner.join(".memex").join("logs");
    Ok(Place {
        owner,
        store,
        in_git_dir: false,
        worktree,
    })
}

/// The folder that owns the store: the main checkout's counterpart of the root (the same path under the main checkout, found from
/// the git common dir), or the root itself when there is no main checkout. The root's path under its toplevel comes from the listing.
fn store_owner(root: &Path, top: Option<&Path>, git: Option<&Git>) -> PathBuf {
    let counterpart = git.and_then(|git| {
        let top = top?;
        let main = main_checkout(git)?;
        root.strip_prefix(top).ok().map(|rel| main.join(rel))
    });
    counterpart.unwrap_or_else(|| root.to_path_buf())
}

/// The main checkout: the parent of a common dir named `.git`. A submodule or a bare repo has none.
fn main_checkout(git: &Git) -> Option<PathBuf> {
    if git.common_dir.file_name() == Some(OsStr::new(".git")) {
        git.common_dir.parent().map(Path::to_path_buf)
    } else {
        None
    }
}

/// The nearest folder with a `.memex/` above `cwd`, within its toplevel (design §4.1 item 1). Outside a repo the walk is unbounded.
/// No git is asked.
pub fn nearest_memex(cwd: &Path) -> Option<PathBuf> {
    let cwd = resolved(cwd);
    let top = toplevel(&cwd);
    nearest_within(&cwd, top.as_deref())
}

/// The root that the read verbs, `show` and `search`, use for `cwd` (design §4.1 item 1): the nearest folder with a `.memex/`
/// inside its toplevel, else the toplevel, else `cwd` itself. No git is asked.
pub fn root_of(cwd: &Path) -> PathBuf {
    let cwd = resolved(cwd);
    let top = toplevel(&cwd);
    nearest_within(&cwd, top.as_deref()).or(top).unwrap_or(cwd)
}

/// The toplevel of the repository holding `cwd` (design §4.1 item 1), canonical and found by listing. None when no `.git` lies
/// above `cwd`. No git is asked, so `init` targets the same folder whether git is missing, failing or present.
pub fn toplevel_of(cwd: &Path) -> Option<PathBuf> {
    toplevel(&resolved(cwd))
}

/// `cwd` as the filesystem spells it, so that its ancestors compare equal with the canonical toplevel. A `cwd` that cannot be
/// resolved is kept as it was given.
fn resolved(cwd: &Path) -> PathBuf {
    dunce::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf())
}

/// True when the store's own folder is listed under its parent by its exact name, as a folder, a file or a symlink. The writes
/// and reads that use a listed store follow it (design §4.4 Name matching).
pub fn store_listed(store: &Path) -> io::Result<bool> {
    match (store.parent(), store.file_name().and_then(OsStr::to_str)) {
        (Some(parent), Some(name)) => lists(parent, name),
        _ => Ok(false),
    }
}

/// True when the nearest `.git` entry at or above `cwd`, found by listing each folder up from `cwd`, is a file: a linked worktree's
/// (design §4.4 git). A folder that cannot be listed holds none, and a symlink is not a file.
fn nearest_git_is_file(cwd: &Path) -> bool {
    cwd.ancestors()
        .find_map(|dir| entry_named(dir, ".git").ok().flatten())
        .and_then(|entry| entry.file_type().ok())
        .is_some_and(|kind| kind.is_file())
}

impl Git {
    /// Asks git about `cwd`. Returns None when git is missing, or `cwd` is not inside a repository. The toplevel is not asked for:
    /// the listing finds it (design §4.1 item 1).
    fn probe(cwd: &Path) -> Option<Git> {
        let dirs = git_lines(cwd, &["rev-parse", "--git-dir", "--git-common-dir"])?;
        let [git_dir, common] = dirs.as_slice() else {
            return None;
        };
        let git_dir = resolve(cwd, git_dir)?;
        let common_dir = resolve(cwd, common)?;
        // Asked from inside the common dir: a worktree of a bare repo answers false from its own cwd.
        let bare = git_line(&common_dir, &["rev-parse", "--is-bare-repository"]).as_deref()
            == Some("true");
        let linked = git_dir != common_dir;
        Some(Git {
            common_dir,
            bare,
            linked,
        })
    }
}

/// Resolves a path git printed, which may be relative to `base`, to a canonical absolute path.
fn resolve(base: &Path, text: &str) -> Option<PathBuf> {
    dunce::canonicalize(base.join(text)).ok()
}

/// The first line git prints for `args` run in `dir`. None when git cannot be started or exits non-zero.
fn git_line(dir: &Path, args: &[&str]) -> Option<String> {
    git_lines(dir, args)?.into_iter().next()
}

/// Every line git prints for `args` run in `dir`. None when git cannot be started or exits non-zero.
fn git_lines(dir: &Path, args: &[&str]) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.lines().map(str::to_string).collect())
}

/// True when git cannot be started, so no `git` is on PATH. Git that starts and then fails is not missing (design §4.4 git).
fn git_missing() -> bool {
    matches!(
        Command::new("git").arg("--version").output(),
        Err(failure) if failure.kind() == io::ErrorKind::NotFound
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn a_store_is_listed_by_its_name_under_its_parent() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let store = dir.path().join(".memex").join("logs");
        assert!(!store_listed(&store).expect("a listing"));
        fs::create_dir_all(&store).expect("a store");
        assert!(store_listed(&store).expect("a listing"));
    }

    #[test]
    fn the_root_is_the_toplevel_when_no_memex_lies_inside_it() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let repo = resolved(dir.path()).join("repo");
        let cwd = repo.join("sub");
        fs::create_dir_all(&cwd).expect("a folder inside the repo");
        fs::create_dir_all(repo.join(".git")).expect("a git folder");
        assert_eq!(root_of(&cwd), repo);
        assert_eq!(toplevel_of(&cwd), Some(repo));
    }
}
