//! Finds where `log` writes, per design §4.1: the root, the folder that owns the store, and the store.
//! The root is the nearest `.memex/` above the cwd, bounded by its toplevel, which one listing of the ancestors finds (layout.rs); else the toplevel, else the cwd. The read verbs use the same root.
//! git answers for the store's place and HEAD while it works; when it gives no answer, the layout (layout.rs) answers from plain files (design §4.4 git). A cwd with no `.memex/` in reach whose toplevel is a `.git` folder or absent is not asked, since git cannot change that answer; `log` gets HEAD's branch in the probe's own call.
//! Cases: DISC-ROOT-01, DISC-ROOT-02, DISC-ROOT-03, DISC-ROOT-04, DISC-ROOT-05, DISC-WT-01, DISC-WT-02, DISC-BARE-01, DISC-GIT-01, DISC-GIT-02, DISC-GIT-03, DISC-NOGIT-01, DISC-NOGIT-02, DISC-NOGIT-03, DISC-NOGIT-04, DISC-NOGIT-05, DISC-NOGIT-06, DISC-NOGIT-07, DISC-TOP-01, DISC-TOP-02, DISC-TOP-03, DISC-TOP-04, DISC-TOP-05, DISC-TOP-06, DISC-TOP-07, DISC-TOP-08, LOG-WRITE-06, LOG-WRITE-10

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::Error;
use crate::layout::{
    git_entry_is_file, git_entry_is_folder, head_branch, holds_memex, layout_of, scan, Layout,
};

pub use crate::layout::{lists, store_listed};

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

/// The branch and short commit of HEAD. Each is None when it cannot be known: the commit needs git, and the branch is None for a
/// detached HEAD.
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

/// Who places the store: git while it answers, else the layout (design §4.4 git).
enum Source {
    /// A repository, as git describes it, or as the layout does for a linked worktree whose common dir is a `.git` folder.
    Repo(Git),
    /// No repository, or a main checkout, submodule or separate git dir: the store is the root's own.
    Own,
    /// A linked worktree whose store the layout cannot place (design §4.4 git).
    Unplaced,
}

/// The store as the reads and `log` see it: the place, and for `log` what HEAD needs.
struct Resolution {
    /// The place, or the `no_root` error when no `.memex/` is in reach, in which case the reads show no store.
    place: Result<Place, Error>,
    /// git's `--abbrev-ref HEAD` line, when `log` asked for HEAD and git answered it.
    abbrev: Option<String>,
    /// The listing's toplevel, and the canonical cwd: where HEAD's layout fallback and its commit are read.
    top: Option<PathBuf>,
    canonical: PathBuf,
}

/// The refusal of `log` in a linked worktree whose store the layout cannot place (design §4.4 git).
const UNPLACED: &str = "cannot find the main checkout's log store: git did not answer and this linked worktree's repository is not a .git folder; run memex log where git works";

/// The probe in one git call: the git dir, the common dir, and whether the common dir is bare (design §4.4 git).
const PROBE: [&str; 4] = [
    "rev-parse",
    "--git-dir",
    "--git-common-dir",
    "--is-bare-repository",
];

/// Finds the place the reads use from `cwd`. A missing `.memex/` is the `no_root` error (exit 3).
pub fn locate(cwd: &Path) -> Result<Place, Error> {
    resolution_of(cwd, false)?.place
}

/// Finds where `log` writes, and HEAD for its entry, from `cwd`. `log` refuses a linked worktree whose store the layout cannot place
/// before anything is written, since its entry would be lost with the worktree (design §4.4 git).
pub fn locate_for_write(cwd: &Path) -> Result<(Place, Head), Error> {
    let found = resolution_of(cwd, true)?;
    let place = found.place?;
    let head = head_of(&found.canonical, found.top.as_deref(), found.abbrev);
    Ok((place, head))
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

/// The folder that owns the store: in a linked worktree, the main checkout's counterpart of the root (the same path under the main
/// checkout, found from the git common dir); else the root itself. The root's path under its toplevel comes from the listing. Git's
/// answer for an outer repo, as under a `.git` folder git skips, is not linked, so the root keeps its own store (DISC-TOP-07).
fn store_owner(root: &Path, top: Option<&Path>, git: Option<&Git>) -> PathBuf {
    let counterpart = git.filter(|git| git.linked).and_then(|git| {
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

/// The store as the cwd's repository places it (design §4.1 item 3): git's answer while git gives one, else the layout's. `for_write`
/// (for `log`) asks HEAD in the probe's call too, and refuses an unplaced linked worktree (design §4.4 git).
fn resolution_of(cwd: &Path, for_write: bool) -> Result<Resolution, Error> {
    let canonical = dunce::canonicalize(cwd)
        .map_err(|failure| Error::io_at("cannot resolve", cwd, &failure))?;
    // One listing of the ancestors gives the toplevel and the root's `.memex/`, the same with git missing, failing or present.
    let (top, found) = scan(&canonical);
    let has_memex = found.is_some();
    let (source, abbrev) = source_of(&canonical, top.as_deref(), has_memex, for_write);
    if for_write && matches!(source, Source::Unplaced) {
        return Err(Error::io_message(UNPLACED.to_string()));
    }
    let worktree = match &source {
        Source::Repo(git) if git.linked => top.clone(),
        _ => None,
    };
    let root = found.or(top.clone()).unwrap_or(canonical.clone());
    let place = match source {
        Source::Repo(git) if git.bare => bare_place(root, &git, has_memex, worktree),
        Source::Repo(git) => checkout_place(root, top.as_deref(), Some(&git), has_memex, worktree),
        Source::Own | Source::Unplaced => {
            checkout_place(root, top.as_deref(), None, has_memex, None)
        }
    };
    Ok(Resolution {
        place,
        abbrev,
        top,
        canonical,
    })
}

/// Who places the store, with git's `--abbrev-ref HEAD` line when the probe asked for it and git gave it.
type Placed = (Source, Option<String>);

/// Who places the store for the repository at `cwd` (design §4.4 git): git while it answers, else the layout. With no `.memex/` in reach
/// and a toplevel that is a `.git` folder or absent, no place can be reached, so git is not asked (DISC-TOP-08 keeps the root's store).
fn source_of(cwd: &Path, top: Option<&Path>, has_memex: bool, head: bool) -> Placed {
    if !has_memex && top.is_none_or(git_entry_is_folder) {
        return (Source::Own, None);
    }
    let (git, abbrev) = Git::probe(cwd, head);
    if let Some(mut git) = git {
        git.linked &= top.is_some_and(git_entry_is_file);
        return (Source::Repo(git), abbrev);
    }
    let source = match top.map(layout_of) {
        Some(Layout::Linked(common_dir)) => Source::Repo(Git {
            common_dir,
            bare: false,
            linked: true,
        }),
        Some(Layout::Unplaced) => Source::Unplaced,
        Some(Layout::Own) | None => Source::Own,
    };
    (source, abbrev)
}

/// The nearest folder with a `.memex/` above `cwd`, within its toplevel (design §4.1 item 1). Outside a repo the walk is unbounded.
/// No git is asked.
pub fn nearest_memex(cwd: &Path) -> Option<PathBuf> {
    scan(&resolved(cwd)).1
}

/// The root that the read verbs, `show` and `search`, use for `cwd` (design §4.1 item 1): the nearest folder with a `.memex/`
/// inside its toplevel, else the toplevel, else `cwd` itself. No git is asked.
pub fn root_of(cwd: &Path) -> PathBuf {
    let cwd = resolved(cwd);
    let (top, found) = scan(&cwd);
    found.or(top).unwrap_or(cwd)
}

/// The toplevel of the repository holding `cwd` (design §4.1 item 1), canonical and found by listing. None when no `.git` lies
/// above `cwd`. No git is asked, so `init` targets the same folder whether git is missing, failing or present.
pub fn toplevel_of(cwd: &Path) -> Option<PathBuf> {
    scan(&resolved(cwd)).0
}

/// `cwd` as the filesystem spells it, so that its ancestors compare equal with the canonical toplevel. A `cwd` that cannot be
/// resolved is kept as it was given.
fn resolved(cwd: &Path) -> PathBuf {
    dunce::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf())
}

/// HEAD of the repository at `canonical`: the branch and short commit that git gave, when it gave the branch (`abbrev`). Else the
/// branch that the layout's `HEAD` names, with no commit (design §4.4 git).
fn head_of(canonical: &Path, top: Option<&Path>, abbrev: Option<String>) -> Head {
    match abbrev {
        Some(name) => Head {
            branch: Some(name).filter(|name| name != "HEAD"),
            commit: git_line(canonical, &["rev-parse", "--short=7", "HEAD"]),
        },
        None => Head {
            branch: top.and_then(head_branch),
            commit: None,
        },
    }
}

impl Git {
    /// Asks git about `cwd`: the probe, and with `head` HEAD's `--abbrev-ref` in the same call. An unborn HEAD fails that call, so the probe
    /// is then asked alone, and HEAD only when the probe answered (one discovery fails both). The toplevel is not asked for (design §4.1).
    fn probe(cwd: &Path, head: bool) -> (Option<Git>, Option<String>) {
        if !head {
            return (Git::probe_alone(cwd), None);
        }
        let args = [&PROBE[..], &["--abbrev-ref", "HEAD"][..]].concat();
        if let Some([dir, common, bare, name]) = git_lines(cwd, &args).as_deref() {
            return (Git::of(cwd, dir, common, bare), Some(name.clone()));
        }
        let git = Git::probe_alone(cwd);
        let abbrev = git
            .as_ref()
            .and_then(|_| git_line(cwd, &["rev-parse", "--abbrev-ref", "HEAD"]));
        (git, abbrev)
    }

    /// The probe in one git call, without HEAD. None when git is missing or fails, or `cwd` is not inside a repository.
    fn probe_alone(cwd: &Path) -> Option<Git> {
        match git_lines(cwd, &PROBE)?.as_slice() {
            [dir, common, bare] => Git::of(cwd, dir, common, bare),
            _ => None,
        }
    }

    /// The git dir and common dir that git printed for `cwd`, resolved. A main checkout (git dir = common dir) keeps the call's bare
    /// answer, which the call inside the repository gives as the common dir does; a linked worktree asks its common dir instead.
    fn of(cwd: &Path, dir: &str, common: &str, bare: &str) -> Option<Git> {
        let git_dir = resolve(cwd, dir)?;
        let common_dir = resolve(cwd, common)?;
        let linked = git_dir != common_dir;
        let bare = if linked {
            git_line(&common_dir, &["rev-parse", "--is-bare-repository"]).as_deref() == Some("true")
        } else {
            bare == "true"
        };
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

#[cfg(test)]
mod tests {
    /// With the git on PATH, `log` from a repo whose `.git` is a FIFO is refused without blocking, and HEAD gives nothing.
    #[cfg(unix)]
    #[test]
    fn a_fifo_git_entry_neither_blocks_nor_places_the_store() {
        use super::*;
        use std::fs;

        let dir = tempfile::tempdir().expect("a temporary folder");
        let repo = resolved(dir.path());
        fs::create_dir_all(repo.join(".memex")).expect("a store owner");
        let made = Command::new("mkfifo")
            .arg(repo.join(".git"))
            .status()
            .expect("mkfifo runs");
        assert!(made.success());
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let outcome = locate_for_write(&repo);
            sender.send((
                outcome.is_err(),
                outcome.ok().and_then(|(_, head)| head.branch),
            ))
        });
        let outcome = receiver.recv_timeout(std::time::Duration::from_secs(5));
        assert_eq!(outcome.expect("locate and head return"), (true, None));
    }

    #[cfg(not(unix))]
    #[test]
    fn a_fifo_git_entry_neither_blocks_nor_places_the_store() {}
}
