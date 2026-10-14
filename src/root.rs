//! Finds where `log` writes, per design §4.1: the root (the nearest `.memex/`, else the git toplevel, else the cwd), the folder whose `.memex/` owns the store, and the store, found through git subprocesses.
//! Cases: DISC-ROOT-01, DISC-ROOT-02, DISC-ROOT-03, DISC-ROOT-04, DISC-WT-01, DISC-WT-02, DISC-BARE-01, DISC-GIT-01, LOG-WRITE-06, LOG-WRITE-10

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::Error;

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
    /// The worktree toplevel. None in a bare repo, which has no worktree.
    toplevel: Option<PathBuf>,
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
        .map_err(|failure| Error::io(&format!("cannot resolve {}", slash(cwd)), &failure))?;
    let found = nearest_memex(&canonical);
    let git = Git::probe(&canonical);
    let toplevel = git.as_ref().and_then(|git| git.toplevel.clone());
    let worktree = git
        .as_ref()
        .filter(|git| git.linked)
        .and_then(|git| git.toplevel.clone());
    let root = found.clone().or(toplevel).unwrap_or(canonical);
    match git {
        Some(git) if git.bare => bare_place(root, &git, found.is_some(), worktree),
        git => checkout_place(root, git.as_ref(), found.is_some(), worktree),
    }
}

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
    git: Option<&Git>,
    has_memex: bool,
    worktree: Option<PathBuf>,
) -> Result<Place, Error> {
    let owner = store_owner(&root, git);
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

/// The folder that owns the store: the main checkout's counterpart of the root (the same path under the
/// main checkout, found from the git common dir), or the root itself when there is no main checkout.
fn store_owner(root: &Path, git: Option<&Git>) -> PathBuf {
    let counterpart = git.and_then(|git| {
        let toplevel = git.toplevel.as_deref()?;
        let main = main_checkout(git)?;
        root.strip_prefix(toplevel).ok().map(|rel| main.join(rel))
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

/// The nearest folder, from `cwd` upward, that holds a `.memex/` directory.
pub fn nearest_memex(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|dir| holds_memex(dir))
        .map(Path::to_path_buf)
}

/// True when `dir` lists a folder named exactly `.memex`. The name is matched against the listing, never probed on a guessed
/// path, so a case-insensitive file system does not match another spelling (design §4.4 Name matching).
fn holds_memex(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|entry| entry.file_name().to_str() == Some(".memex") && entry.path().is_dir())
    })
}

impl Git {
    /// Asks git about `cwd`. Returns None when git is missing, or `cwd` is not inside a repository.
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
        let toplevel =
            git_line(cwd, &["rev-parse", "--show-toplevel"]).and_then(|top| resolve(cwd, &top));
        let linked = git_dir != common_dir;
        Some(Git {
            toplevel,
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
