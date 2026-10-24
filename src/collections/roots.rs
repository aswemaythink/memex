//! Keeps the manifest roots a reader may walk, each with the folder it is read from (design §4.2).
//! It also parses the optional `roots` key. A root that resolves outside the repo is left out with its warning, so no reader walks it. A root the repo lacks is read from the main checkout's copy, with its warning, when a linked worktree has one.
//! Cases: MANK-ROOTS-01, MANK-WT-01, MANK-WT-02, MANK-WT-03, MANK-WT-04, MANK-WT-05, MAN-LOAD-09, MAN-LOAD-10, MAN-ROOTS-01

use std::cell::OnceCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value as Json};

use super::borrow;
use super::manifest::Manifest;
use crate::error;
use crate::root::slash;
use crate::walk::rel_string;

/// A root that a reader walks: `rel` is its root-relative folder, and `home` is the folder that `rel` is joined to. The home is the
/// repo root, or the main checkout's copy of the root for a root that a linked worktree lacks (design §4.2 Roots in a linked worktree).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    /// The root-relative folder, with `/` separators; empty for the root itself.
    pub rel: String,
    /// The folder the root is read from, which `rel` is joined to.
    pub home: PathBuf,
}

/// The roots of `manifest` that a reader walks, in order, each with its home. A root that resolves outside the repo is left out with
/// `<collection>: root <root> leads outside the repo; skipped`, so no reader walks it (design §4.2 Unreadable files). A root with no
/// entry of any kind at its path is read from the main checkout's copy when a linked worktree holds a folder there, with a warning
/// that names that copy; a root that neither checkout holds is kept and reads nothing.
pub fn inside_repo(manifest: &Manifest, repo: &Path, warnings: &mut Vec<String>) -> Vec<Root> {
    // The main checkout is looked up once, and only when some root is missing from the repo.
    let counterpart = OnceCell::new();
    manifest
        .roots
        .iter()
        .filter_map(|rel| {
            let home = home_of(&manifest.name, rel, repo, &counterpart, warnings)?;
            Some(Root {
                rel: rel.clone(),
                home,
            })
        })
        .collect()
}

/// The folder that the root `rel` is read from, or None when it is left out with its warning (design §4.2).
fn home_of(
    name: &str,
    rel: &str,
    repo: &Path,
    counterpart: &OnceCell<Option<PathBuf>>,
    warnings: &mut Vec<String>,
) -> Option<PathBuf> {
    let path = repo.join(rel);
    if is_missing(&path) {
        return borrowed_home(name, rel, repo, counterpart, warnings);
    }
    if resolves_outside(&path, repo) {
        warnings.push(outside_warning(name, rel));
        return None;
    }
    Some(repo.to_path_buf())
}

/// The main checkout's copy of a root that the repo lacks. It is the home when that copy is a folder, and it is left out with its
/// warning when the copy resolves outside the counterpart root (design §4.2 Roots in a linked worktree). Otherwise the root is kept
/// with the repo as its home, where it reads nothing.
fn borrowed_home(
    name: &str,
    rel: &str,
    repo: &Path,
    counterpart: &OnceCell<Option<PathBuf>>,
    warnings: &mut Vec<String>,
) -> Option<PathBuf> {
    let Some(main) = counterpart
        .get_or_init(|| borrow::counterpart(repo))
        .as_deref()
        .filter(|main| main.join(rel).is_dir())
    else {
        return Some(repo.to_path_buf());
    };
    let copy = main.join(rel);
    if resolves_outside(&copy, main) {
        warnings.push(outside_warning(name, rel));
        return None;
    }
    warnings.push(error::warning(&format!(
        "{name}: root {rel} is not in this worktree; read from the main checkout {}",
        slash(&copy)
    )));
    Some(main.to_path_buf())
}

/// True when no entry of any kind lies at `path`, which is what `symlink_metadata` reporting NotFound means. A dangling symlink is an
/// entry, so it is not missing (design §4.2 Roots in a linked worktree).
fn is_missing(path: &Path) -> bool {
    fs::symlink_metadata(path).is_err_and(|failure| failure.kind() == io::ErrorKind::NotFound)
}

/// True when `base` resolves, through its symbolic links, to a folder outside `repo`. Both are canonicalised with `dunce`, so a
/// symlink inside the repo stays inside. A path that cannot be resolved is not judged: the walk reads what it can (design §4.2).
fn resolves_outside(base: &Path, repo: &Path) -> bool {
    match (dunce::canonicalize(base), dunce::canonicalize(repo)) {
        (Ok(folder), Ok(home)) => !folder.starts_with(home),
        _ => false,
    }
}

/// The warning for a root that leads outside its folder, which is then left out (design §4.2 Unreadable files).
fn outside_warning(name: &str, rel: &str) -> String {
    error::warning(&format!(
        "{name}: root {rel} leads outside the repo; skipped"
    ))
}

/// The explicit roots, each a folder inside the root, written relative to it; none when the key is absent, since READMEs may mark
/// them (design §4.2 Roots).
pub(super) fn roots_of(object: &Map<String, Json>) -> Result<Vec<String>, String> {
    match object.get("roots") {
        Some(Json::Array(items)) => items.iter().map(root_of).collect(),
        Some(_) => Err("key 'roots' must be a list of folders".to_string()),
        None => Ok(Vec::new()),
    }
}

/// One root, normalised to `/` separators; an absolute root, or one that climbs with `..`, is refused.
fn root_of(item: &Json) -> Result<String, String> {
    let text = item
        .as_str()
        .ok_or("key 'roots' must be a list of folders")?;
    rel_string(Path::new(text))
        .ok_or_else(|| format!("root '{text}' is not a folder inside the root"))
}

#[cfg(test)]
mod tests {
    use crate::collections::manifest::Kind;
    use crate::collections::testing;

    use super::{inside_repo, Root};

    /// The root-relative folders of `roots`, in order.
    fn names(roots: &[Root]) -> Vec<&str> {
        roots.iter().map(|root| root.rel.as_str()).collect()
    }

    /// A root that does not exist is kept, since its walk reads nothing, and it gives no warning (MAN-LOAD-01).
    #[test]
    fn a_missing_root_is_kept_with_no_warning() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let manifest = testing::manifest(Kind::Folder, &["missing"]);
        let mut warnings = Vec::new();
        let roots = inside_repo(&manifest, dir.path(), &mut warnings);
        assert_eq!(names(&roots), ["missing"]);
        assert_eq!(roots[0].home, dir.path());
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// MANK-ROOTS-01: a root that is a symlink to a folder outside the repo is left out with its warning, and the roots beside it
    /// stay. A symlink inside the repo to a folder inside it is kept.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_root_outside_the_repo_is_left_out_with_its_warning() {
        use std::fs;
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().expect("a temporary folder");
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join("units")).expect("a folder");
        fs::create_dir_all(dir.path().join("outside")).expect("a folder");
        symlink(dir.path().join("outside"), repo.join("linked")).expect("a symlink");
        symlink("units", repo.join("shortcut")).expect("a symlink");
        let manifest = testing::manifest(Kind::Folder, &["linked", "shortcut", "units"]);
        let mut warnings = Vec::new();
        let usable = inside_repo(&manifest, &repo, &mut warnings);
        assert_eq!(names(&usable), ["shortcut", "units"]);
        assert_eq!(
            warnings,
            ["memex: warning: cases: root linked leads outside the repo; skipped"]
        );
    }
}
